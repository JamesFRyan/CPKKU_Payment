//! A deterministic, in-memory [`PaymentProvider`] for tests and local wiring.
//!
//! It performs **no network I/O**. Its outputs are a pure function of the
//! request and the configured [`MockBehavior`], so tests stay reproducible.
//! This is the "Mock Provider" required alongside the provider abstraction
//! (dev prompt section 12 / start-command task 14); it is not a stand-in for
//! the real KKU Payment adapter, which lands in Phase 3.

use async_trait::async_trait;
use chrono::{TimeZone, Utc};

use super::{
    CallbackVerificationRequest, CancelPaymentRequest, CancelPaymentResult, CreatePaymentRequest,
    CreatePaymentResult, PaymentMethod, PaymentProvider, PaymentStatusRequest, PaymentStatusResult,
    ProviderCapabilities, ProviderError, ProviderEvent, ProviderPaymentStatus,
    ProviderRefundStatus, RefundPaymentRequest, RefundPaymentResult, RefundStatusRequest,
    RefundStatusResult, VoidPaymentRequest, VoidPaymentResult,
};

/// How the mock should respond, so tests can exercise both the happy path and
/// error classification without a real provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MockBehavior {
    /// Every operation succeeds deterministically.
    #[default]
    Succeed,
    /// Every operation fails with [`ProviderError::Timeout`] — used to exercise
    /// the indeterminate-outcome path (dev prompt section 18).
    FailTimeout,
}

/// A deterministic in-memory payment provider.
#[derive(Debug, Clone, Default)]
pub struct MockProvider {
    behavior: MockBehavior,
}

impl MockProvider {
    /// A mock whose every operation succeeds.
    pub fn new() -> Self {
        Self {
            behavior: MockBehavior::Succeed,
        }
    }

    /// A mock with the given behavior.
    pub fn with_behavior(behavior: MockBehavior) -> Self {
        Self { behavior }
    }

    /// Returns the configured timeout error for `operation`, or `Ok(())` when
    /// the mock is set to succeed.
    fn gate(&self, operation: &str) -> Result<(), ProviderError> {
        match self.behavior {
            MockBehavior::Succeed => Ok(()),
            MockBehavior::FailTimeout => Err(ProviderError::Timeout {
                operation: operation.to_string(),
            }),
        }
    }
}

#[async_trait]
impl PaymentProvider for MockProvider {
    async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<CreatePaymentResult, ProviderError> {
        self.gate("create_payment")?;
        // QR/Barcode return a payload to render; Credit does not.
        let payment_payload = match request.method {
            PaymentMethod::Qr | PaymentMethod::Barcode => {
                Some(format!("MOCK-PAYLOAD-{}", request.payment_id))
            }
            PaymentMethod::Credit => None,
        };
        Ok(CreatePaymentResult {
            provider_reference: format!("MOCK-{}", request.payment_id),
            status: ProviderPaymentStatus::Pending,
            payment_payload,
            expires_at: None,
        })
    }

    async fn get_payment_status(
        &self,
        request: PaymentStatusRequest,
    ) -> Result<PaymentStatusResult, ProviderError> {
        self.gate("get_payment_status")?;
        Ok(PaymentStatusResult {
            provider_reference: request.provider_reference,
            status: ProviderPaymentStatus::Paid,
            paid_amount: None,
            paid_at: None,
        })
    }

    async fn cancel_payment(
        &self,
        request: CancelPaymentRequest,
    ) -> Result<CancelPaymentResult, ProviderError> {
        self.gate("cancel_payment")?;
        Ok(CancelPaymentResult {
            provider_reference: request.provider_reference,
            status: ProviderPaymentStatus::Cancelled,
        })
    }

    async fn void_payment(
        &self,
        request: VoidPaymentRequest,
    ) -> Result<VoidPaymentResult, ProviderError> {
        self.gate("void_payment")?;
        Ok(VoidPaymentResult {
            provider_reference: request.provider_reference,
            status: ProviderPaymentStatus::Voided,
        })
    }

    async fn refund_payment(
        &self,
        request: RefundPaymentRequest,
    ) -> Result<RefundPaymentResult, ProviderError> {
        self.gate("refund_payment")?;
        Ok(RefundPaymentResult {
            provider_refund_reference: format!("MOCK-RF-{}", request.refund_id),
            status: ProviderRefundStatus::Succeeded,
        })
    }

    async fn get_refund_status(
        &self,
        request: RefundStatusRequest,
    ) -> Result<RefundStatusResult, ProviderError> {
        self.gate("get_refund_status")?;
        Ok(RefundStatusResult {
            provider_refund_reference: request.provider_refund_reference,
            status: ProviderRefundStatus::Succeeded,
        })
    }

    async fn verify_callback(
        &self,
        request: CallbackVerificationRequest,
    ) -> Result<ProviderEvent, ProviderError> {
        self.gate("verify_callback")?;
        // The mock treats an empty body as an unverifiable callback so tests can
        // exercise the rejection path deterministically.
        if request.raw_body.is_empty() {
            return Err(ProviderError::CallbackVerification(
                "empty body".to_string(),
            ));
        }
        Ok(ProviderEvent {
            provider_reference: "MOCK-CALLBACK".to_string(),
            status: ProviderPaymentStatus::Paid,
            // Fixed epoch instant keeps the mock deterministic.
            occurred_at: Utc.timestamp_opt(0, 0).single().expect("epoch is valid"),
        })
    }

    async fn get_capabilities(&self) -> Result<ProviderCapabilities, ProviderError> {
        self.gate("get_capabilities")?;
        Ok(ProviderCapabilities {
            supported_methods: vec![
                PaymentMethod::Qr,
                PaymentMethod::Barcode,
                PaymentMethod::Credit,
            ],
            supports_refund: true,
            supports_void: true,
            supports_cancel: true,
        })
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        self.gate("health_check")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{Money, RequestContext};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn ctx() -> RequestContext {
        RequestContext {
            tenant_id: Uuid::nil(),
            correlation_id: Uuid::nil(),
        }
    }

    fn create_request(method: PaymentMethod) -> CreatePaymentRequest {
        CreatePaymentRequest {
            context: ctx(),
            payment_id: Uuid::nil(),
            amount: Money {
                amount: Decimal::new(10000, 2),
                currency: "THB".to_string(),
            },
            method,
            description: None,
            return_url: None,
        }
    }

    #[tokio::test]
    async fn create_payment_succeeds_and_returns_qr_payload() {
        let provider = MockProvider::new();
        let result = provider
            .create_payment(create_request(PaymentMethod::Qr))
            .await
            .expect("mock should succeed");
        assert_eq!(result.status, ProviderPaymentStatus::Pending);
        assert!(result.payment_payload.is_some(), "QR must carry a payload");
    }

    #[tokio::test]
    async fn credit_payment_has_no_payload() {
        let provider = MockProvider::new();
        let result = provider
            .create_payment(create_request(PaymentMethod::Credit))
            .await
            .expect("mock should succeed");
        assert!(result.payment_payload.is_none());
    }

    #[tokio::test]
    async fn timeout_behavior_is_classified_as_timeout() {
        let provider = MockProvider::with_behavior(MockBehavior::FailTimeout);
        let err = provider
            .create_payment(create_request(PaymentMethod::Qr))
            .await
            .expect_err("should time out");
        assert!(err.is_timeout(), "error must classify as a timeout");
    }

    #[tokio::test]
    async fn empty_callback_body_is_rejected() {
        let provider = MockProvider::new();
        let err = provider
            .verify_callback(CallbackVerificationRequest {
                raw_body: Vec::new(),
                headers: Vec::new(),
            })
            .await
            .expect_err("empty body should fail verification");
        assert!(!err.is_timeout());
    }
}
