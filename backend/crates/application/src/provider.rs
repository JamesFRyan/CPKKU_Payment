//! Provider abstraction (dev prompt section 12).
//!
//! The [`PaymentProvider`] port is the *only* way Payment Core talks to an
//! external payment provider — the core must never call a provider's HTTP API
//! directly. Concrete adapters (e.g. `provider-kku-payment`) implement this
//! trait; provider-specific request/response models stay inside those adapter
//! crates and must never leak here or into `domain` (Provider Contract Rule
//! #14).
//!
//! The types below are deliberately provider-neutral. They establish the shape
//! Payment Core codes against; exact fields will be refined as Payment Core
//! (Phase 5) firms up, but no field here may encode KKU-Payment-specific
//! behavior.

pub mod mock;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

/// A monetary amount together with its ISO-4217 currency code.
///
/// Money is always carried as a [`Decimal`] (never a float) per the financial
/// correctness rule (dev prompt section 10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    pub amount: Decimal,
    /// ISO-4217 alphabetic code, e.g. `"THB"`.
    pub currency: String,
}

/// Provider-neutral payment method. Adapters map these onto whatever the
/// concrete provider supports (and reject the ones it does not, via
/// [`ProviderCapabilities`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentMethod {
    Qr,
    Barcode,
    Credit,
}

/// Provider-neutral lifecycle status for a payment. This is *not* the domain
/// payment state machine (that lives in `domain`); it is the normalized view of
/// what a provider reports, which the core maps onto its own states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderPaymentStatus {
    Pending,
    Authorized,
    Paid,
    Cancelled,
    Voided,
    Failed,
    Expired,
}

/// Provider-neutral lifecycle status for a refund.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderRefundStatus {
    Pending,
    Succeeded,
    Failed,
}

/// Correlation identifiers threaded through every provider call so requests can
/// be traced end-to-end and mapped back to a tenant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestContext {
    pub tenant_id: Uuid,
    /// Correlation/trace id propagated to the provider where the contract
    /// allows it.
    pub correlation_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePaymentRequest {
    pub context: RequestContext,
    /// Our payment id (UUIDv7, dev prompt section 11). Used for idempotency and
    /// correlation with the provider reference.
    pub payment_id: Uuid,
    pub amount: Money,
    pub method: PaymentMethod,
    pub description: Option<String>,
    /// Where the payer is returned after an interactive flow, when applicable.
    pub return_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePaymentResult {
    /// The provider's own reference for this payment.
    pub provider_reference: String,
    pub status: ProviderPaymentStatus,
    /// Present for QR/Barcode methods: the payload to render for the payer.
    pub payment_payload: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentStatusRequest {
    pub context: RequestContext,
    pub payment_id: Uuid,
    pub provider_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentStatusResult {
    pub provider_reference: String,
    pub status: ProviderPaymentStatus,
    pub paid_amount: Option<Money>,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelPaymentRequest {
    pub context: RequestContext,
    pub payment_id: Uuid,
    pub provider_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelPaymentResult {
    pub provider_reference: String,
    pub status: ProviderPaymentStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoidPaymentRequest {
    pub context: RequestContext,
    pub payment_id: Uuid,
    pub provider_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoidPaymentResult {
    pub provider_reference: String,
    pub status: ProviderPaymentStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundPaymentRequest {
    pub context: RequestContext,
    pub payment_id: Uuid,
    pub provider_reference: String,
    /// Our refund id (UUIDv7), used for provider-side idempotency.
    pub refund_id: Uuid,
    pub amount: Money,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundPaymentResult {
    pub provider_refund_reference: String,
    pub status: ProviderRefundStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundStatusRequest {
    pub context: RequestContext,
    pub refund_id: Uuid,
    pub provider_refund_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundStatusResult {
    pub provider_refund_reference: String,
    pub status: ProviderRefundStatus,
}

/// A callback delivered by the provider, handed to [`PaymentProvider::verify_callback`]
/// for signature verification and normalization.
///
/// The `raw_body` is the exact bytes received. Signature verification must run
/// against these unmodified bytes — never a re-serialized copy (dev prompt
/// section 21, Callback Raw Body Rule).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallbackVerificationRequest {
    pub raw_body: Vec<u8>,
    /// Provider-supplied headers relevant to verification (e.g. signature,
    /// nonce). Adapter decides which it needs; the port stays neutral.
    pub headers: Vec<(String, String)>,
}

/// A verified, normalized provider event derived from a callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderEvent {
    pub provider_reference: String,
    pub status: ProviderPaymentStatus,
    pub occurred_at: DateTime<Utc>,
}

/// What a provider can actually do, so the core can reject unsupported
/// operations up front rather than failing mid-flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCapabilities {
    pub supported_methods: Vec<PaymentMethod>,
    pub supports_refund: bool,
    pub supports_void: bool,
    pub supports_cancel: bool,
}

/// Errors a provider operation can fail with, normalized across providers.
///
/// [`ProviderError::Timeout`] is deliberately distinct from every other
/// variant: a timeout means the outcome is *unknown* (the request may or may
/// not have taken effect), so later phases must classify it separately and run
/// inquiry-before-retry rather than blindly retrying (dev prompt sections 18 &
/// 19). Use [`ProviderError::is_timeout`] to branch on that.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// The request timed out; the outcome is indeterminate and must not be
    /// blindly retried.
    #[error("provider timed out during `{operation}`")]
    Timeout { operation: String },

    /// The provider rejected the request as invalid (4xx-class, deterministic —
    /// retrying the same request will not help).
    #[error("provider rejected `{operation}`: {message}")]
    Rejected { operation: String, message: String },

    /// The provider returned a transient failure that may succeed on retry
    /// (5xx-class, connection reset, etc.).
    #[error("transient provider failure during `{operation}`: {message}")]
    Transient { operation: String, message: String },

    /// A callback failed signature/verification and must be discarded.
    #[error("callback verification failed: {0}")]
    CallbackVerification(String),

    /// The provider does not support the requested operation (see
    /// [`ProviderCapabilities`]).
    #[error("operation `{0}` not supported by this provider")]
    Unsupported(String),
}

impl ProviderError {
    /// True only for [`ProviderError::Timeout`], whose outcome is indeterminate.
    pub fn is_timeout(&self) -> bool {
        matches!(self, ProviderError::Timeout { .. })
    }
}

/// The central payment-provider port (dev prompt section 12).
///
/// Payment Core depends on this trait, never on a concrete provider. Adapters
/// implement it; all provider-specific models stay inside the adapter crate.
#[async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<CreatePaymentResult, ProviderError>;

    async fn get_payment_status(
        &self,
        request: PaymentStatusRequest,
    ) -> Result<PaymentStatusResult, ProviderError>;

    async fn cancel_payment(
        &self,
        request: CancelPaymentRequest,
    ) -> Result<CancelPaymentResult, ProviderError>;

    async fn void_payment(
        &self,
        request: VoidPaymentRequest,
    ) -> Result<VoidPaymentResult, ProviderError>;

    async fn refund_payment(
        &self,
        request: RefundPaymentRequest,
    ) -> Result<RefundPaymentResult, ProviderError>;

    async fn get_refund_status(
        &self,
        request: RefundStatusRequest,
    ) -> Result<RefundStatusResult, ProviderError>;

    async fn verify_callback(
        &self,
        request: CallbackVerificationRequest,
    ) -> Result<ProviderEvent, ProviderError>;

    async fn get_capabilities(&self) -> Result<ProviderCapabilities, ProviderError>;

    async fn health_check(&self) -> Result<(), ProviderError>;
}
