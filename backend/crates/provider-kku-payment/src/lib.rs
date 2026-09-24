#![forbid(unsafe_code)]

//! KKU Payment provider adapter (Provider Code: `KKU_PAYMENT`).
//!
//! Implements the `PaymentProvider` port against the KKU Payment API
//! (see `api-payment kku.json`, OpenAPI 3.1, version 2.0.0). Provider-generated
//! models must stay in this crate and must never leak into the domain model
//! (Provider Contract Rule #14). Module breakdown (adapter/client/auth/signer/
//! etc., dev prompt section 13) lands in Phase 3 once the open questions in
//! the repo README are resolved with the provider team.
