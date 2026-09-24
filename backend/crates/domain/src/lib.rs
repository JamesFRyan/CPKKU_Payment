#![forbid(unsafe_code)]

//! Domain layer: business entities, value objects, and rules for CPKKU Payment.
//!
//! Must not depend on Axum, SQLx, Redis, Lapin, Reqwest, or any KKU Payment
//! provider schema (dev prompt section 5, Provider Contract Rule #14).
