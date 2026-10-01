//! Payment Core use cases (dev prompt Phase 5).
//!
//! Orchestration that sits on top of the [`PaymentProvider`](crate::provider)
//! port and the `domain` state machine, with financial idempotency (section
//! 15). There is no I/O here: concrete stores live in `infrastructure`, and
//! provider HTTP lives in `provider-kku-payment`.

pub mod create_payment;
pub mod idempotency;
