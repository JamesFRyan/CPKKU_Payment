//! Payment state machine (dev prompt section 14).
//!
//! This is the *authoritative internal* payment lifecycle for Payment Core. It
//! is deliberately independent of any provider's own status vocabulary: a
//! provider status must be mapped onto one of these internal states by the
//! adapter's `status_mapper` (dev prompt section 13), never used directly
//! (section 14, "Provider Status ห้ามใช้เป็น Internal Status โดยตรง").
//!
//! The crate owns two things that downstream layers depend on:
//! - [`PaymentStatus`], the 15-state enum, and its allowed-transition guard
//!   ([`PaymentStatus::can_transition_to`] / [`PaymentStatus::transition`]).
//! - [`PaymentTransition`], the audit record every transition must capture.
//!
//! The transition *edges* below are not drawn explicitly in the dev prompt; they
//! are inferred from the payment lifecycle and locked down by the unit tests at
//! the bottom of this file. An invalid transition is rejected
//! ([`InvalidTransition`]), never silently applied.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Internal payment lifecycle status (dev prompt section 14, 15 states).
///
/// Serialized as `SCREAMING_SNAKE_CASE` to match the names used across the dev
/// prompt, the `payment_status_history` table, and audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Created,
    Pending,
    AwaitingPayment,
    Processing,
    Authorized,
    Paid,
    Failed,
    Expired,
    Cancelled,
    VoidPending,
    Voided,
    RefundPending,
    PartiallyRefunded,
    Refunded,
    ManualReview,
}

impl PaymentStatus {
    /// Terminal states have no outgoing transitions: the payment's lifecycle is
    /// finished and its financial record must never be hard-deleted (dev prompt
    /// section 25).
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            PaymentStatus::Voided
                | PaymentStatus::Refunded
                | PaymentStatus::Failed
                | PaymentStatus::Expired
                | PaymentStatus::Cancelled
        )
    }

    /// The explicitly allowed next states for a given current state.
    ///
    /// In addition to the edges listed here, any non-terminal state may move to
    /// [`PaymentStatus::ManualReview`] — that escape hatch is applied in
    /// [`PaymentStatus::can_transition_to`], not duplicated in every arm. It
    /// models an ambiguous or unknown provider status, which section 14 requires
    /// to route to `MANUAL_REVIEW` rather than be guessed.
    fn allowed_next(self) -> &'static [PaymentStatus] {
        use PaymentStatus::*;
        match self {
            Created => &[Pending, AwaitingPayment, Cancelled, Failed],
            Pending => &[
                AwaitingPayment,
                Processing,
                Authorized,
                Paid,
                Failed,
                Expired,
                Cancelled,
            ],
            AwaitingPayment => &[Processing, Authorized, Paid, Expired, Cancelled, Failed],
            Processing => &[Authorized, Paid, Failed, Expired],
            Authorized => &[Paid, VoidPending, Failed, Expired, Cancelled],
            Paid => &[VoidPending, RefundPending],
            VoidPending => &[Voided, Failed],
            RefundPending => &[PartiallyRefunded, Refunded, Failed],
            PartiallyRefunded => &[RefundPending, Refunded],
            // Manual resolution of a flagged payment into a settled outcome.
            ManualReview => &[Paid, Failed, Voided, Refunded, Cancelled],
            // Terminal states: no outgoing edges.
            Voided | Refunded | Failed | Expired | Cancelled => &[],
        }
    }

    /// Whether moving from `self` to `to` is a legal transition.
    ///
    /// A self-transition (`self == to`) is not a transition and is rejected.
    /// Any non-terminal state may transition to [`PaymentStatus::ManualReview`].
    pub fn can_transition_to(self, to: PaymentStatus) -> bool {
        if self == to {
            return false;
        }
        if to == PaymentStatus::ManualReview {
            return !self.is_terminal();
        }
        self.allowed_next().contains(&to)
    }

    /// Validate and build the audit record for a transition from `self` to the
    /// new status in `details`.
    ///
    /// Returns [`InvalidTransition`] if the edge is not allowed, so callers
    /// cannot persist an illegal state change.
    pub fn transition(
        self,
        to: PaymentStatus,
        details: TransitionDetails,
    ) -> Result<PaymentTransition, InvalidTransition> {
        if !self.can_transition_to(to) {
            return Err(InvalidTransition { from: self, to });
        }
        Ok(PaymentTransition {
            previous_status: self,
            new_status: to,
            provider_status: details.provider_status,
            provider_status_code: details.provider_status_code,
            source: details.source,
            actor: details.actor,
            reason: details.reason,
            correlation_id: details.correlation_id,
            timestamp: details.timestamp,
        })
    }
}

/// What triggered a status transition (the `Source` field of section 14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransitionSource {
    /// A synchronous provider API response (e.g. create/status inquiry).
    Provider,
    /// An inbound, verified provider callback.
    Callback,
    /// A tenant-facing API request.
    Api,
    /// A human operator action (back office).
    Operator,
    /// An automated internal process (e.g. expiry sweep).
    System,
    /// A reconciliation batch correcting state against the provider.
    Reconciliation,
}

/// The mutable, caller-supplied fields of a transition. Together with the
/// `previous_status` (the receiver of [`PaymentStatus::transition`]) and the
/// derived `new_status`, these make up the nine fields section 14 requires every
/// transition to record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionDetails {
    /// The raw provider status that drove this change, if any. `None` for
    /// internally-initiated transitions (e.g. an expiry sweep).
    pub provider_status: Option<String>,
    /// The provider's status/result code, if any.
    pub provider_status_code: Option<String>,
    pub source: TransitionSource,
    /// Who/what performed the action (operator id, worker name, etc.).
    pub actor: String,
    pub reason: Option<String>,
    pub correlation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// An audit record of a single, validated status transition.
///
/// Mirrors the `payment_status_history` row (dev prompt sections 14 & 24): the
/// nine fields that must be captured on every transition. Instances are only
/// created by [`PaymentStatus::transition`], so an illegal edge can never be
/// recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentTransition {
    pub previous_status: PaymentStatus,
    pub new_status: PaymentStatus,
    pub provider_status: Option<String>,
    pub provider_status_code: Option<String>,
    pub source: TransitionSource,
    pub actor: String,
    pub reason: Option<String>,
    pub correlation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Returned when a requested status transition is not permitted by the state
/// machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid payment status transition from {from:?} to {to:?}")]
pub struct InvalidTransition {
    pub from: PaymentStatus,
    pub to: PaymentStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const ALL_STATES: [PaymentStatus; 15] = [
        PaymentStatus::Created,
        PaymentStatus::Pending,
        PaymentStatus::AwaitingPayment,
        PaymentStatus::Processing,
        PaymentStatus::Authorized,
        PaymentStatus::Paid,
        PaymentStatus::Failed,
        PaymentStatus::Expired,
        PaymentStatus::Cancelled,
        PaymentStatus::VoidPending,
        PaymentStatus::Voided,
        PaymentStatus::RefundPending,
        PaymentStatus::PartiallyRefunded,
        PaymentStatus::Refunded,
        PaymentStatus::ManualReview,
    ];

    fn details() -> TransitionDetails {
        TransitionDetails {
            provider_status: None,
            provider_status_code: None,
            source: TransitionSource::System,
            actor: "test".to_string(),
            reason: None,
            correlation_id: Uuid::nil(),
            timestamp: Utc.timestamp_opt(0, 0).unwrap(),
        }
    }

    #[test]
    fn happy_path_qr_lifecycle_is_allowed() {
        use PaymentStatus::*;
        let path = [Created, AwaitingPayment, Processing, Paid];
        for pair in path.windows(2) {
            assert!(
                pair[0].can_transition_to(pair[1]),
                "{:?} -> {:?} should be allowed",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn refund_lifecycle_is_allowed() {
        use PaymentStatus::*;
        assert!(Paid.can_transition_to(RefundPending));
        assert!(RefundPending.can_transition_to(PartiallyRefunded));
        assert!(PartiallyRefunded.can_transition_to(RefundPending));
        assert!(PartiallyRefunded.can_transition_to(Refunded));
        assert!(RefundPending.can_transition_to(Refunded));
    }

    #[test]
    fn void_lifecycle_is_allowed() {
        use PaymentStatus::*;
        assert!(Paid.can_transition_to(VoidPending));
        assert!(VoidPending.can_transition_to(Voided));
    }

    #[test]
    fn illegal_transitions_are_rejected() {
        use PaymentStatus::*;
        // Cannot jump straight from Created to Paid without going through the flow.
        assert!(!Created.can_transition_to(Paid));
        // Cannot refund something that was never paid.
        assert!(!Created.can_transition_to(RefundPending));
        // Cannot resurrect a terminal state.
        assert!(!Refunded.can_transition_to(Paid));
    }

    #[test]
    fn self_transition_is_rejected() {
        for s in ALL_STATES {
            assert!(!s.can_transition_to(s), "{s:?} -> {s:?} must be rejected");
        }
    }

    #[test]
    fn terminal_states_have_no_outgoing_edges() {
        for s in ALL_STATES.into_iter().filter(|s| s.is_terminal()) {
            for to in ALL_STATES {
                assert!(
                    !s.can_transition_to(to),
                    "terminal {s:?} must not transition to {to:?}"
                );
            }
        }
    }

    #[test]
    fn unknown_status_routes_to_manual_review_from_any_non_terminal_state() {
        for s in ALL_STATES {
            // A state already in ManualReview is excluded: that would be a
            // (rejected) self-transition, not an escalation.
            let expected = !s.is_terminal() && s != PaymentStatus::ManualReview;
            assert_eq!(
                s.can_transition_to(PaymentStatus::ManualReview),
                expected,
                "{s:?} -> ManualReview should be {expected}"
            );
        }
    }

    #[test]
    fn manual_review_can_be_resolved() {
        use PaymentStatus::*;
        for to in [Paid, Failed, Voided, Refunded, Cancelled] {
            assert!(ManualReview.can_transition_to(to));
        }
    }

    #[test]
    fn transition_builds_record_on_valid_edge() {
        let t = PaymentStatus::Paid
            .transition(PaymentStatus::RefundPending, details())
            .expect("valid edge");
        assert_eq!(t.previous_status, PaymentStatus::Paid);
        assert_eq!(t.new_status, PaymentStatus::RefundPending);
    }

    #[test]
    fn transition_errors_on_invalid_edge() {
        let err = PaymentStatus::Created
            .transition(PaymentStatus::Paid, details())
            .unwrap_err();
        assert_eq!(err.from, PaymentStatus::Created);
        assert_eq!(err.to, PaymentStatus::Paid);
    }

    #[test]
    fn status_serializes_as_screaming_snake_case() {
        let json = serde_json::to_string(&PaymentStatus::AwaitingPayment).unwrap();
        assert_eq!(json, "\"AWAITING_PAYMENT\"");
        let parsed: PaymentStatus = serde_json::from_str("\"MANUAL_REVIEW\"").unwrap();
        assert_eq!(parsed, PaymentStatus::ManualReview);
    }
}
