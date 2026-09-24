# CP-KKU Payment Gateway

Multi-tenant payment orchestration platform for Khon Kaen University, sitting
between tenant applications and the external **KKU Payment** provider
(`unipay.kku.ac.th`). Tenant applications must never call KKU Payment directly.

## Layout

- `backend/` — Rust 1.98.1 / Axum / SQLx Cargo workspace.
- `docs/api-payment kku.json` — KKU Payment provider OpenAPI 3.1 contract (source of truth for the provider adapter).
- `docs/API Manaulมออกใบเสร็จ-KKU.docx` — KKU ERP (Oracle Fusion) receipt integration manual.
- `docs/CP-KKU_PaymentGateway_Project_Master.docx` — business/architecture blueprint.
- `docs/CPKKU_Payment_Master_Development_Prompt_v3_Rust_DockerCompose_Receipt.md` — authoritative Rust development prompt. Supersedes the Go/Gin/Kubernetes/Vault stack described in the Project Master document; the team confirmed Rust + Docker Compose is what's actually being built.

## Status

**Phase 1 — Rust workspace foundation.** Crate/app boundaries only, no business
logic yet (that starts at Phase 2, Provider Abstraction).

Rust is not installed on this machine yet. Before building:

```bash
# install rustup, then:
cd backend
cargo build
```

`Cargo.lock` is intentionally not committed yet — generate it with the pinned
toolchain and commit it once `cargo build` succeeds (Dependency Rule, dev
prompt section 6).

## Open questions against the KKU Payment contract

Tracked ahead of Phase 3 (KKU Payment Adapter) — do not guess these, confirm
with the provider/ERP teams first (Provider Contract Rule #10-11):

1. No callback/webhook payload schema or registration mechanism is documented
   in `api-payment kku.json`, despite the dev prompt requiring raw-body
   signature verification on provider callbacks.
2. The `x-pay-sign-nonce` signing header is mentioned in the OpenAPI tag
   description but its algorithm/fields are never defined.
3. No credit/debit card endpoint exists in the KKU Payment OpenAPI spec, only
   QR/Barcode and Debts — but Phase 7 requires Credit Payment.
4. `POST /api/kku/v1/debts` has no `Idempotency-Key` support documented.
5. Rate limits / IP allowlist for KKU Payment are undocumented.
6. ERP OAuth `scope` value is line-wrapped/ambiguous in the source manual;
   confirm the exact string with the ERP team.
7. `CreateMiscReceipt` production endpoint is not provided (SIT/UAT only).
8. No ERP receipt inquiry API exists, so ambiguous `CreateMiscReceipt` timeouts
   must always route to `MANUAL_REVIEW` (never blind-retried).
