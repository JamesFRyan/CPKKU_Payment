//! The create-payment use case (dev prompt Phase 5).
//!
//! Orchestrates the [`PaymentProvider`] port, normalizes the provider-neutral
//! status onto the `domain` state machine, and guarantees financial idempotency
//! via an [`IdempotencyStore`]. On a provider timeout the outcome is
//! indeterminate (section 18): the error propagates *without* recording
//! idempotency, so the operation stays safely retryable.

use std::sync::Arc;

use chrono::Utc;
use domain::payment::{
    InvalidTransition, PaymentStatus, PaymentTransition, TransitionDetails, TransitionSource,
};
use uuid::Uuid;

use super::idempotency::{IdempotencyCheck, IdempotencyError, IdempotencyScope, IdempotencyStore};
use crate::provider::{
    CreatePaymentRequest, Money, PaymentMethod, PaymentProvider, ProviderError,
    ProviderPaymentStatus, RequestContext,
};

const OPERATION: &str = "CREATE_PAYMENT";

/// Input to the create-payment use case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePaymentCommand {
    pub tenant_id: Uuid,
    pub application_id: Uuid,
    pub correlation_id: Uuid,
    /// Our payment id (UUIDv7, dev prompt section 11).
    pub payment_id: Uuid,
    /// Client-supplied idempotency key for this operation (section 15).
    pub idempotency_key: String,
    pub amount: Money,
    pub method: PaymentMethod,
    pub description: Option<String>,
    pub return_url: Option<String>,
}

impl CreatePaymentCommand {
    /// A stable identity of the request payload — the "Request Hash" of section
    /// 15. Two commands under the same idempotency key must produce the same
    /// fingerprint to be treated as a replay; otherwise it is a conflict.
    fn fingerprint(&self) -> String {
        format!(
            "{}|{}|{}|{:?}",
            self.payment_id, self.amount.amount, self.amount.currency, self.method
        )
    }
}

/// The recorded result of a create-payment operation; also the snapshot stored
/// for idempotent replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePaymentOutcome {
    pub payment_id: Uuid,
    /// The provider's own reference for this payment.
    pub provider_reference: String,
    /// The normalized domain status after creation.
    pub status: PaymentStatus,
    /// QR/Barcode payload to render for the payer, when applicable.
    pub payment_payload: Option<String>,
    /// The initial `CREATED -> status` transition record.
    pub initial_transition: PaymentTransition,
}

#[derive(Debug, thiserror::Error)]
pub enum CreatePaymentError {
    #[error(transparent)]
    Idempotency(#[from] IdempotencyError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    /// The provider reported a status that is not a legal initial state for a
    /// freshly created payment. Surfaced at the provider boundary rather than
    /// silently coerced.
    #[error("provider returned an invalid initial status")]
    UnexpectedProviderStatus(#[from] InvalidTransition),
}

/// Normalize the provider-neutral [`ProviderPaymentStatus`] onto the internal
/// [`PaymentStatus`].
///
/// This maps the *already provider-neutral* port enum; it is NOT the
/// KKU-specific `status_mapper` (that lives in the `provider-kku-payment`
/// adapter and is blocked on the provider contract, Phase 3).
pub fn normalize_provider_status(status: ProviderPaymentStatus) -> PaymentStatus {
    match status {
        ProviderPaymentStatus::Pending => PaymentStatus::Pending,
        ProviderPaymentStatus::Authorized => PaymentStatus::Authorized,
        ProviderPaymentStatus::Paid => PaymentStatus::Paid,
        ProviderPaymentStatus::Cancelled => PaymentStatus::Cancelled,
        ProviderPaymentStatus::Voided => PaymentStatus::Voided,
        ProviderPaymentStatus::Failed => PaymentStatus::Failed,
        ProviderPaymentStatus::Expired => PaymentStatus::Expired,
    }
}

/// The create-payment use case, generic over the idempotency store.
pub struct CreatePayment<S> {
    provider: Arc<dyn PaymentProvider>,
    idempotency: Arc<S>,
}

impl<S> CreatePayment<S>
where
    S: IdempotencyStore<CreatePaymentOutcome>,
{
    pub fn new(provider: Arc<dyn PaymentProvider>, idempotency: Arc<S>) -> Self {
        Self {
            provider,
            idempotency,
        }
    }

    pub async fn execute(
        &self,
        command: CreatePaymentCommand,
    ) -> Result<CreatePaymentOutcome, CreatePaymentError> {
        let scope = IdempotencyScope {
            tenant_id: command.tenant_id,
            application_id: command.application_id,
            operation: OPERATION.to_string(),
            idempotency_key: command.idempotency_key.clone(),
        };
        let fingerprint = command.fingerprint();

        // Replay a previous response for the same key + payload; a same-key /
        // different-payload request surfaces as IdempotencyError::Conflict here.
        if let IdempotencyCheck::Replay(previous) =
            self.idempotency.check(&scope, &fingerprint).await?
        {
            return Ok(previous);
        }

        // Not seen before: perform the provider call. A timeout propagates
        // without recording anything, leaving the operation retryable.
        let result = self
            .provider
            .create_payment(CreatePaymentRequest {
                context: RequestContext {
                    tenant_id: command.tenant_id,
                    correlation_id: command.correlation_id,
                },
                payment_id: command.payment_id,
                amount: command.amount.clone(),
                method: command.method,
                description: command.description.clone(),
                return_url: command.return_url.clone(),
            })
            .await?;

        let status = normalize_provider_status(result.status);
        let initial_transition = PaymentStatus::Created.transition(
            status,
            TransitionDetails {
                provider_status: Some(format!("{:?}", result.status)),
                provider_status_code: None,
                source: TransitionSource::Provider,
                actor: "payment-core".to_string(),
                reason: Some(OPERATION.to_string()),
                correlation_id: command.correlation_id,
                timestamp: Utc::now(),
            },
        )?;

        let outcome = CreatePaymentOutcome {
            payment_id: command.payment_id,
            provider_reference: result.provider_reference,
            status,
            payment_payload: result.payment_payload,
            initial_transition,
        };

        self.idempotency
            .record(&scope, &fingerprint, outcome.clone())
            .await?;
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::super::idempotency::InMemoryIdempotencyStore;
    use super::*;
    use crate::provider::CallbackVerificationRequest;
    use crate::provider::mock::{MockBehavior, MockProvider};
    use crate::provider::{
        CancelPaymentRequest, CancelPaymentResult, CreatePaymentResult, PaymentStatusRequest,
        PaymentStatusResult, ProviderCapabilities, ProviderEvent, RefundPaymentRequest,
        RefundPaymentResult, RefundStatusRequest, RefundStatusResult, VoidPaymentRequest,
        VoidPaymentResult,
    };
    use async_trait::async_trait;
    use rust_decimal::Decimal;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Wraps a [`MockProvider`] and counts `create_payment` calls, so tests can
    /// prove a replay does NOT re-invoke the provider.
    struct CountingProvider {
        inner: MockProvider,
        create_calls: AtomicUsize,
    }

    impl CountingProvider {
        fn new(behavior: MockBehavior) -> Self {
            Self {
                inner: MockProvider::with_behavior(behavior),
                create_calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl PaymentProvider for CountingProvider {
        async fn create_payment(
            &self,
            request: CreatePaymentRequest,
        ) -> Result<CreatePaymentResult, ProviderError> {
            self.create_calls.fetch_add(1, Ordering::SeqCst);
            self.inner.create_payment(request).await
        }
        async fn get_payment_status(
            &self,
            request: PaymentStatusRequest,
        ) -> Result<PaymentStatusResult, ProviderError> {
            self.inner.get_payment_status(request).await
        }
        async fn cancel_payment(
            &self,
            request: CancelPaymentRequest,
        ) -> Result<CancelPaymentResult, ProviderError> {
            self.inner.cancel_payment(request).await
        }
        async fn void_payment(
            &self,
            request: VoidPaymentRequest,
        ) -> Result<VoidPaymentResult, ProviderError> {
            self.inner.void_payment(request).await
        }
        async fn refund_payment(
            &self,
            request: RefundPaymentRequest,
        ) -> Result<RefundPaymentResult, ProviderError> {
            self.inner.refund_payment(request).await
        }
        async fn get_refund_status(
            &self,
            request: RefundStatusRequest,
        ) -> Result<RefundStatusResult, ProviderError> {
            self.inner.get_refund_status(request).await
        }
        async fn verify_callback(
            &self,
            request: CallbackVerificationRequest,
        ) -> Result<ProviderEvent, ProviderError> {
            self.inner.verify_callback(request).await
        }
        async fn get_capabilities(&self) -> Result<ProviderCapabilities, ProviderError> {
            self.inner.get_capabilities().await
        }
        async fn health_check(&self) -> Result<(), ProviderError> {
            self.inner.health_check().await
        }
    }

    fn command(key: &str, minor_units: i64) -> CreatePaymentCommand {
        CreatePaymentCommand {
            tenant_id: Uuid::nil(),
            application_id: Uuid::nil(),
            correlation_id: Uuid::nil(),
            payment_id: Uuid::nil(),
            idempotency_key: key.to_string(),
            amount: Money {
                amount: Decimal::new(minor_units, 2),
                currency: "THB".to_string(),
            },
            method: PaymentMethod::Qr,
            description: None,
            return_url: None,
        }
    }

    #[tokio::test]
    async fn create_payment_succeeds_and_records_initial_transition() {
        let use_case = CreatePayment::new(
            Arc::new(MockProvider::new()),
            Arc::new(InMemoryIdempotencyStore::new()),
        );
        let outcome = use_case
            .execute(command("key-1", 10000))
            .await
            .expect("create should succeed");

        assert_eq!(outcome.status, PaymentStatus::Pending);
        assert!(outcome.payment_payload.is_some(), "QR carries a payload");
        assert_eq!(
            outcome.initial_transition.previous_status,
            PaymentStatus::Created
        );
        assert_eq!(
            outcome.initial_transition.new_status,
            PaymentStatus::Pending
        );
    }

    #[tokio::test]
    async fn same_key_same_payload_replays_without_calling_provider_again() {
        let provider = Arc::new(CountingProvider::new(MockBehavior::Succeed));
        let use_case =
            CreatePayment::new(provider.clone(), Arc::new(InMemoryIdempotencyStore::new()));

        let first = use_case.execute(command("key-1", 10000)).await.unwrap();
        let second = use_case.execute(command("key-1", 10000)).await.unwrap();

        assert_eq!(first, second, "replay must return the stored outcome");
        assert_eq!(
            provider.create_calls.load(Ordering::SeqCst),
            1,
            "provider must be invoked only once"
        );
    }

    #[tokio::test]
    async fn same_key_different_payload_is_conflict() {
        let use_case = CreatePayment::new(
            Arc::new(MockProvider::new()),
            Arc::new(InMemoryIdempotencyStore::new()),
        );
        use_case.execute(command("key-1", 10000)).await.unwrap();

        let err = use_case
            .execute(command("key-1", 20000))
            .await
            .expect_err("different payload under same key must conflict");
        assert!(matches!(
            err,
            CreatePaymentError::Idempotency(IdempotencyError::Conflict)
        ));
    }

    #[tokio::test]
    async fn provider_timeout_is_not_recorded_and_stays_retryable() {
        let provider = Arc::new(CountingProvider::new(MockBehavior::FailTimeout));
        let store = Arc::new(InMemoryIdempotencyStore::new());
        let use_case = CreatePayment::new(provider.clone(), store.clone());

        let err = use_case
            .execute(command("key-1", 10000))
            .await
            .expect_err("timeout should surface");
        assert!(matches!(
            err,
            CreatePaymentError::Provider(ref e) if e.is_timeout()
        ));

        // Nothing was recorded, so the same key is still free to retry.
        match store
            .check(
                &IdempotencyScope {
                    tenant_id: Uuid::nil(),
                    application_id: Uuid::nil(),
                    operation: OPERATION.to_string(),
                    idempotency_key: "key-1".to_string(),
                },
                "whatever",
            )
            .await
            .unwrap()
        {
            IdempotencyCheck::New => {}
            IdempotencyCheck::Replay(_) => panic!("timeout must not record idempotency"),
        }
    }
}
