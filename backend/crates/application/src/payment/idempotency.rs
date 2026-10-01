//! Financial idempotency port (dev prompt section 15).
//!
//! Idempotency for financial mutations MUST be backed by PostgreSQL (section
//! 15); Redis may only be an optional cache/lock and must never be the source of
//! truth. This module defines the provider-neutral *port* plus an in-memory
//! implementation for tests and local wiring. The durable PostgreSQL adapter
//! lands in `infrastructure`.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use uuid::Uuid;

/// The uniqueness scope of an idempotent operation (section 15):
/// `tenant_id + application_id + operation + idempotency_key`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyScope {
    pub tenant_id: Uuid,
    pub application_id: Uuid,
    /// The operation name, e.g. `"CREATE_PAYMENT"`.
    pub operation: String,
    pub idempotency_key: String,
}

/// Result of checking a scope before performing work.
pub enum IdempotencyCheck<T> {
    /// No prior record: the caller owns this key, should perform the work, then
    /// call [`IdempotencyStore::record`].
    New,
    /// A prior record with a matching request fingerprint: return this stored
    /// response instead of repeating the work (section 15, "Same Key + Same
    /// Payload => Return previous response").
    Replay(T),
}

#[derive(Debug, thiserror::Error)]
pub enum IdempotencyError {
    /// Same key, different payload (section 15, "Same Key + Different Payload =>
    /// 409 Conflict").
    #[error("idempotency key reused with a different request payload")]
    Conflict,
    /// The underlying store failed.
    #[error("idempotency store backend error: {0}")]
    Backend(String),
}

/// Store for idempotent operation results, keyed by [`IdempotencyScope`].
///
/// The `request_fingerprint` is a stable identity of the request payload (the
/// "Request Hash" of section 15). The in-memory implementation compares it
/// directly; the PostgreSQL implementation may hash it. `T` is the response
/// snapshot the caller wants replayed.
#[async_trait]
pub trait IdempotencyStore<T>: Send + Sync
where
    T: Clone + Send + Sync,
{
    /// Look up `scope`. Returns [`IdempotencyError::Conflict`] when a record
    /// exists whose fingerprint differs from `request_fingerprint`.
    async fn check(
        &self,
        scope: &IdempotencyScope,
        request_fingerprint: &str,
    ) -> Result<IdempotencyCheck<T>, IdempotencyError>;

    /// Persist the response for `scope` after the work succeeded.
    async fn record(
        &self,
        scope: &IdempotencyScope,
        request_fingerprint: &str,
        response: T,
    ) -> Result<(), IdempotencyError>;
}

/// In-memory [`IdempotencyStore`] for tests and local wiring.
///
/// Not durable — the authoritative store is PostgreSQL (section 15). This lets
/// Payment Core be exercised against the mock provider without a database.
pub struct InMemoryIdempotencyStore<T> {
    records: Mutex<HashMap<IdempotencyScope, (String, T)>>,
}

impl<T> InMemoryIdempotencyStore<T> {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
        }
    }
}

impl<T> Default for InMemoryIdempotencyStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> IdempotencyStore<T> for InMemoryIdempotencyStore<T>
where
    T: Clone + Send + Sync,
{
    async fn check(
        &self,
        scope: &IdempotencyScope,
        request_fingerprint: &str,
    ) -> Result<IdempotencyCheck<T>, IdempotencyError> {
        let records = self
            .records
            .lock()
            .map_err(|_| IdempotencyError::Backend("lock poisoned".to_string()))?;
        match records.get(scope) {
            None => Ok(IdempotencyCheck::New),
            Some((fingerprint, response)) => {
                if fingerprint == request_fingerprint {
                    Ok(IdempotencyCheck::Replay(response.clone()))
                } else {
                    Err(IdempotencyError::Conflict)
                }
            }
        }
    }

    async fn record(
        &self,
        scope: &IdempotencyScope,
        request_fingerprint: &str,
        response: T,
    ) -> Result<(), IdempotencyError> {
        let mut records = self
            .records
            .lock()
            .map_err(|_| IdempotencyError::Backend("lock poisoned".to_string()))?;
        records.insert(scope.clone(), (request_fingerprint.to_string(), response));
        Ok(())
    }
}
