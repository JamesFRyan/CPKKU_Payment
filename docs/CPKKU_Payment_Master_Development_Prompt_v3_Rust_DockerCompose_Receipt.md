# CPKKU Payment — MASTER DEVELOPMENT PROMPT v2

## Multi-Tenant Payment Orchestration Platform

### Backend: Rust 1.98.1 Stable | Provider: KKU Payment | Deployment: Docker Compose Only

---

# 0. บทบาทของทีมพัฒนา

ให้ทำหน้าที่เป็นทีมพัฒนาระดับ Senior ประกอบด้วย:

- Enterprise / Solution Architect
- Payment System Architect
- Senior Rust Backend Engineer
- Senior React / Next.js Engineer
- PostgreSQL Database Engineer
- RabbitMQ Integration Engineer
- DevSecOps Engineer
- Application Security Engineer
- QA Automation Engineer
- Performance Engineer
- Technical Writer

ภารกิจคือออกแบบ พัฒนา Refactor ทดสอบ และจัดเตรียมระบบ **CPKKU Payment** สำหรับใช้งาน Production จริง

ระบบต้องเป็น **Multi-Tenant Payment Orchestration Platform** และเชื่อมต่อ Provider ภายนอกชื่อ **KKU Payment**

Code ที่ส่งมอบต้อง:

- Compile และ Run ได้
- ใช้ Rust Stable ที่ Pin Version
- มี Error Handling และ Validation
- มี Structured Logging / Tracing
- มี Automated Tests
- มี Database Migration
- มี OpenAPI
- มี Security Controls
- มี Documentation
- ไม่มี Secret ใน Source Code
- รองรับ Docker Compose Production
- รองรับ Audit และ Observability
- ป้องกัน Duplicate Financial Transaction

ห้ามสร้างเพียง Demo หรือ Pseudo Code ยกเว้น Phase ที่ระบุให้สร้าง Skeleton ก่อน

---

# 1. Provider Manual — Source of Truth

ใช้คู่มือ Provider ต่อไปนี้เป็นแหล่งอ้างอิงหลัก:

**KKU Payment Provider Manual**

https://unipay.kku.ac.th/docs/v1/kku#description/introduction

ค่ามาตรฐาน:

```text
Provider Name : KKU Payment
Provider Code : KKU_PAYMENT
Adapter Name  : kku_payment
```

## Provider Contract Rules

1. ต้องอ่าน Provider Manual ก่อน Implement Integration จริง
2. Endpoint ต้องตรงตามคู่มือ
3. HTTP Method ต้องตรงตามคู่มือ
4. Authentication ต้องตรงตามคู่มือ
5. Header ต้องตรงตามคู่มือ
6. Signature Algorithm ต้องตรงตามคู่มือ
7. Request/Response Schema ต้องตรงตามคู่มือ
8. Callback Format และ Callback ACK ต้องตรงตามคู่มือ
9. Provider Status/Error Code ต้องตรงตามคู่มือ
10. ห้ามเดา Provider Contract
11. หากข้อมูลไม่ชัดเจน ให้สร้าง TODO และ Open Question
12. หากมี OpenAPI JSON/YAML ให้เก็บ Snapshot ไว้ใน Repository
13. ต้องมี Contract Test สำหรับ Provider
14. Provider-generated Model ห้ามรั่วเข้า Domain Model
15. Provider Authentication ต้องเป็น Contract-driven
16. ห้าม Implement OAuth2, Basic Auth, mTLS หรือ Signing แบบใดแบบหนึ่งเพียงเพราะ Architecture รองรับ ต้องเปิดใช้เฉพาะ Mechanism ที่คู่มือ KKU Payment ยืนยัน
17. ต้องบันทึกวันที่/เวอร์ชันของ Provider Contract ที่ใช้พัฒนา

---

# 2. Architecture Constraint

โครงการนี้ใช้:

```text
Docker Compose Only
```

ไม่ใช้:

```text
Kubernetes
Helm
Vault
HA Cluster
Automatic Failover
Disaster Recovery Site
Multi-Site DR
```

Operational Backup/Restore ยังคงต้องมี

---

# 3. Technology Stack

## Frontend

- React
- Next.js
- TypeScript
- App Router
- Tailwind CSS
- shadcn/ui หรือ Radix UI
- TanStack Query
- TanStack Table
- React Hook Form
- Zod
- next-intl
- ECharts/Recharts
- Vitest
- Playwright

## Backend

```text
Rust             : 1.98.1 Stable
Rust Edition     : 2024
Async Runtime    : Tokio
Web Framework    : Axum
Middleware       : Tower / Tower HTTP
Database         : PostgreSQL
Database Client  : SQLx
Cache            : Redis
Message Broker   : RabbitMQ
RabbitMQ Client  : Lapin
HTTP Client      : Reqwest
TLS              : rustls
Serialization    : Serde / serde_json
Validation       : validator
Money            : rust_decimal
UUID             : uuid
Errors           : thiserror + anyhow
Logging          : tracing
Telemetry        : OpenTelemetry
Metrics          : Prometheus
OpenAPI          : utoipa / OpenAPI 3.1
```

## Rust Toolchain

`rust-toolchain.toml`

```toml
[toolchain]
channel = "1.98.1"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

Production ห้ามใช้ Nightly

Rust Upgrade ต้องผ่าน Regression, Integration, Contract และ Performance Test ก่อนเสมอ

---

# 4. Target Architecture

```text
Tenant Application / ERP / Web / Mobile
                   |
                   v
             Nginx / TLS
                   |
          +--------+---------+
          |                  |
          v                  v
   Next.js Frontend     Rust Payment API
                       Axum + Tower
                             |
        +--------------------+----------------------+
        |                    |                      |
        v                    v                      v
   PostgreSQL              Redis                RabbitMQ
 Source of Truth       Cache/Optimization      Async Events
        |                                           |
        |                                           v
        |                                   Rust Worker Services
        |                                           |
        +-----------------------------+-------------+
                                      |
                                      v
                              KKU Payment Adapter
                          Reqwest + Rustls + Serde
                                      |
                                      v
                                 KKU Payment
```

หลักการ:

```text
CPKKU Payment = Payment Orchestration Platform
KKU Payment   = External Payment Provider
```

Tenant Application ห้ามเชื่อม KKU Payment โดยตรง

---

# 5. Backend Architecture

ใช้:

```text
Modular Monolith + Independent Rust Worker Processes
```

Domain Modules:

- Identity
- Tenant
- Merchant
- Application
- Payment
- Provider
- Refund
- Void
- Webhook
- Reconciliation
- Settlement
- Approval
- Audit
- Risk
- Notification
- Report

Domain Layer ห้าม Dependency โดยตรงกับ:

- Axum
- SQLx
- Redis
- RabbitMQ/Lapin
- Reqwest
- KKU Payment schema

ใช้ Ports/Traits ระหว่าง Domain/Application กับ Infrastructure

---

# 6. Rust Cargo Workspace Structure

```text
backend/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
│
├── crates/
│   ├── domain/
│   ├── application/
│   ├── infrastructure/
│   ├── provider-kku-payment/
│   └── api/
│
├── apps/
│   ├── payment-api/
│   ├── payment-worker/
│   ├── kku-payment-worker/
│   ├── callback-worker/
│   ├── webhook-worker/
│   ├── refund-worker/
│   ├── reconciliation-worker/
│   ├── audit-worker/
│   ├── outbox-publisher/
│   └── scheduler/
│
├── migrations/
├── tests/
├── benches/
└── docker/
```

`Cargo.lock` ต้อง Commit เข้า Repository

---

# 7. Rust Dependency Rules

Workspace ต้องมี crate ที่จำเป็น เช่น:

```text
tokio
axum
tower
tower-http
serde
serde_json
sqlx
redis
lapin
reqwest
rustls
rust_decimal
uuid
chrono/time
thiserror
anyhow
tracing
tracing-subscriber
opentelemetry
validator
utoipa
async-trait
```

หากใช้:

```rust
#[async_trait::async_trait]
```

ต้องประกาศ `async-trait` ชัดเจน

Provider Registry ที่รองรับ Dynamic Provider ให้ใช้รูปแบบประมาณ:

```text
Arc<dyn PaymentProvider + Send + Sync>
```

ห้ามใช้ Version `*`

ต้องควบคุม Dependency ผ่าน `Cargo.lock`

---

# 8. Multi-Tenant Model

```text
Platform
 └── Tenant
      └── Merchant
           └── Application
                ├── API Credential
                ├── Webhook
                ├── Allowed IP
                ├── Rate Limit
                ├── Payment Policy
                └── Security Policy
```

Tenant Isolation ต้องมี:

- Tenant-aware Authentication
- Tenant-aware Authorization
- Tenant Scope ใน API Client
- Tenant Scope ใน Role
- Tenant ID ใน Business Entity
- PostgreSQL Row-Level Security
- Tenant-specific Webhook Secret
- Tenant-specific Credential
- Tenant-specific Report Scope
- Cross-Tenant Detection
- Cross-Tenant Security Audit

ห้ามเชื่อถือ `tenant_id` จาก Request Body เป็น Source of Truth

Tenant ต้อง resolve จาก Trusted Identity Context

---

# 9. PostgreSQL RLS + SQLx Pool — Critical Requirement

ห้ามใช้ session-scoped Tenant Context บน pooled connection เช่น:

```sql
SET app.tenant_id = '...';
```

โดยไม่มี Transaction Boundary

Tenant-scoped Database Operation ทุกครั้งต้องทำ:

```text
Acquire SQLx Connection
        |
        v
BEGIN
        |
        v
Set Transaction-local Tenant Context
        |
        v
Execute Tenant Queries
        |
        v
COMMIT / ROLLBACK
        |
        v
Return Connection to Pool
```

ใช้แนวทาง:

```sql
SELECT set_config(
    'app.tenant_id',
    $1,
    true
);
```

หรือ `SET LOCAL` ที่เทียบเท่า

ข้อกำหนด:

```text
All tenant-scoped database operations MUST run
inside a PostgreSQL transaction with transaction-local
tenant context.

Never retain tenant context at PostgreSQL session level
on pooled connections.
```

ต้องมี Automated Test สำหรับ:

- Connection reuse
- Cross-Tenant attempt
- Transaction rollback
- Concurrent tenants

---

# 10. Financial Correctness

PostgreSQL เป็น Source of Truth สำหรับ Financial Correctness

ห้ามให้ Redis เป็นตัวรับประกันความถูกต้องหลัก

## Money

ห้ามใช้:

```text
f32
f64
```

ใช้:

```text
rust_decimal::Decimal
```

Database:

```sql
NUMERIC(18,2)
```

สำหรับ final THB amount

Calculation ที่มี rate/fee/tax ใช้ precision สูงกว่า เช่น:

```text
rate              NUMERIC(12,8)
calculated_amount NUMERIC(20,6)
final_amount      NUMERIC(18,2)
```

ต้องกำหนด Finance-approved Rounding Policy เช่น:

```text
Currency    : THB
Minor Unit  : 2
Rounding    : Finance-approved rule
```

Developer ห้ามเลือก Rounding Mode เอง

---

# 11. UUID Policy

ใช้ UUID v7

ID ใหม่ต้องสร้างจาก Rust Application Layer

ตัวอย่าง:

```rust
let payment_id = Uuid::now_v7();
```

ห้ามใช้ UUID function ที่สร้าง UUID version อื่นโดยไม่ตั้งใจสำหรับ Entity ที่กำหนดให้เป็น UUID v7

---

# 12. Provider Abstraction

สร้าง Provider Interface กลาง

```rust
#[async_trait::async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn create_payment(&self, request: CreatePaymentRequest) -> Result<CreatePaymentResult, ProviderError>;
    async fn get_payment_status(&self, request: PaymentStatusRequest) -> Result<PaymentStatusResult, ProviderError>;
    async fn cancel_payment(&self, request: CancelPaymentRequest) -> Result<CancelPaymentResult, ProviderError>;
    async fn void_payment(&self, request: VoidPaymentRequest) -> Result<VoidPaymentResult, ProviderError>;
    async fn refund_payment(&self, request: RefundPaymentRequest) -> Result<RefundPaymentResult, ProviderError>;
    async fn get_refund_status(&self, request: RefundStatusRequest) -> Result<RefundStatusResult, ProviderError>;
    async fn verify_callback(&self, request: CallbackVerificationRequest) -> Result<ProviderEvent, ProviderError>;
    async fn get_capabilities(&self) -> Result<ProviderCapabilities, ProviderError>;
    async fn health_check(&self) -> Result<(), ProviderError>;
}
```

Payment Core ห้ามเรียก KKU Payment HTTP API โดยตรง

---

# 13. KKU Payment Adapter

```text
crates/provider-kku-payment/src/
├── adapter.rs
├── client.rs
├── auth.rs
├── token_manager.rs
├── signer.rs
├── request_mapper.rs
├── response_mapper.rs
├── status_mapper.rs
├── error_mapper.rs
├── callback.rs
├── capability.rs
├── health.rs
├── config.rs
└── models.rs
```

Adapter รับผิดชอบ:

- Provider Authentication ตาม Contract
- Secure HTTP Client
- Request Signing ตาม Contract
- Request/Response Mapping
- Provider Status Mapping
- Provider Error Mapping
- Callback Verification
- Capability Mapping
- Health Check
- Timeout Classification
- Correlation/Trace Propagation

Generated/Provider Models อยู่ใน Adapter เท่านั้น

---

# 14. Payment State Machine

Internal Status:

```text
CREATED
PENDING
AWAITING_PAYMENT
PROCESSING
AUTHORIZED
PAID
FAILED
EXPIRED
CANCELLED
VOID_PENDING
VOIDED
REFUND_PENDING
PARTIALLY_REFUNDED
REFUNDED
MANUAL_REVIEW
```

Provider Status ห้ามใช้เป็น Internal Status โดยตรง

Unknown Provider Status:

```text
MANUAL_REVIEW
```

ทุก Transition ต้องเก็บ:

- Previous Status
- New Status
- Provider Status
- Provider Status Code
- Source
- Actor
- Reason
- Correlation ID
- Timestamp

---

# 15. Idempotency — Critical Requirement

Financial Idempotency ต้องเก็บใน PostgreSQL

ตาราง:

```text
idempotency_records
```

Unique Constraint:

```text
tenant_id
application_id
operation
idempotency_key
```

ต้องเก็บ:

- Request Hash
- Response Snapshot
- HTTP Status
- Created At
- Expires At

Behavior:

```text
Same Key + Same Payload
=> Return previous response

Same Key + Different Payload
=> 409 Conflict
```

Redis ใช้เป็น:

- Optional cache
- Optional contention optimization
- Optional short-lived lock

Redis ห้ามเป็น Source of Truth ของ Idempotency

ระบบต้องยังป้องกัน Duplicate Payment ได้แม้ Redis restart

---

# 16. Redis Correctness Rule

Redis lock ห้ามเป็น Financial Correctness Mechanism

Refund/Balance-sensitive operation ใช้ PostgreSQL transaction + row locking

```text
BEGIN
 |
SELECT payment ... FOR UPDATE
 |
Calculate refundable balance
 |
Validate refund
 |
INSERT refund
 |
COMMIT
```

หลัก:

```text
PostgreSQL constraints/locks = correctness
Redis                         = optimization
```

---

# 17. Create Payment Execution Model

## Create Payment = Synchronous Provider Call

ใช้เมื่อ Client ต้องได้ QR / Redirect URL ทันที

```text
Client
  |
POST /payments
  |
  v
Validate + Idempotency
  |
  v
Create Internal Payment
  |
COMMIT
  |
  v
KKU Payment Adapter
  |
  v
KKU Payment
  |
  v
Store Provider Result
  |
  v
Return QR / Redirect Session
```

RabbitMQ ใช้สำหรับ:

- Payment Inquiry
- Timeout Recovery
- Callback post-processing
- Tenant Webhook Delivery
- Refund asynchronous work
- Reconciliation
- Settlement import
- Report generation
- Notifications
- Audit/SIEM forwarding

---

# 18. Provider Timeout Handling — Critical Requirement

กรณี Create Payment ส่งถึง Provider แล้ว Timeout:

```text
Provider Request Sent
       |
       v
Timeout / Unknown Result
       |
       v
DO NOT mark FAILED
       |
       v
Set PROCESSING
       |
       v
Enqueue Payment Inquiry
       |
       v
KKU Payment Inquiry
       |
   +---+----------------+
   |                    |
 FOUND                NOT FOUND
   |                    |
Update Status      Retry Create only
                  according to safe policy
```

ห้าม Retry Create Payment ทันทีโดยไม่ Inquiry ก่อน

Unknown outcome ที่ resolve ไม่ได้:

```text
MANUAL_REVIEW
```

---

# 19. Retry Matrix

ต้องแยก Retry ตาม Operation

Safe Retry เช่น Inquiry/Health Check

Unsafe/Financial Mutation เช่น Create Payment, Refund, Void, Capture ต้องใช้:

- Idempotency
- Provider Reference
- Inquiry-before-retry
- Explicit retry classification

Retry ต้องมี:

- Maximum attempts
- Exponential backoff
- Jitter
- Timeout
- Retry-After support เมื่อ Provider ระบุ

ห้าม blind retry validation/auth/permission error

---

# 20. Callback Architecture — Persist Before Business Processing

```text
KKU Payment
    |
    v
Callback API
    |
Capture exact raw body bytes
    |
Verify minimum provider-required security
    |
Compute payload hash
    |
BEGIN
    |
Persist callback/inbox record
    |
COMMIT
    |
Return Provider-required ACK
    |
    v
Async Callback Worker
    |
Business Processing
    |
Payment State Transition
    |
Audit + Outbox
```

- ACK format/status code ต้องยึด Provider Manual
- หาก Contract กำหนด verify signature ก่อน ACK ให้ทำตาม Contract
- Business processing ที่ใช้เวลานานไม่ควรอยู่ใน synchronous callback path

---

# 21. Callback Raw Body Rule — Critical Requirement

หาก Signature คำนวณจาก raw HTTP body:

```text
HTTP Raw Bytes
   |
   +--> Signature Verification
   +--> SHA-256 Hash
   +--> Deserialize JSON
```

ห้าม:

```text
JSON -> Rust Struct -> Serialize -> Verify Signature
```

Raw body ต้องเก็บเท่าที่จำเป็นตาม Security/PDPA policy

หากมี Sensitive Data ให้เก็บ Hash, Masked Payload หรือ Encrypted Payload ตาม requirement

---

# 22. RabbitMQ Architecture — Single Node

Requirement ปัจจุบัน:

```text
No RabbitMQ HA Cluster
```

ใช้ Single-node RabbitMQ พร้อม:

- Durable Exchanges
- Durable Queues
- Persistent Messages
- Publisher Confirms
- Manual ACK
- Retry Queues
- DLQ
- Persistent Named Volume
- Monitoring

ไม่กำหนด 3-node cluster และไม่บังคับ quorum HA design

Reliability หลักมาจาก Transactional Outbox + Inbox

---

# 23. Transactional Outbox

ห้าม Publish ก่อน Database Commit

```text
BEGIN
 |
Business Transaction
 |
Status History
 |
Audit
 |
Outbox Event
 |
COMMIT
 |
Outbox Publisher
 |
RabbitMQ
```

ถ้า RabbitMQ Down ให้ `outbox_event` คงสถานะ pending และ publish ใหม่เมื่อ RabbitMQ กลับมา

---

# 24. Inbox / Idempotent Consumer

```text
Receive message
 |
Validate schema/version
 |
Check processed_messages
 |
BEGIN
 |
Business logic
 |
Insert processed_messages
 |
COMMIT
 |
ACK
```

ACK หลัง Commit เท่านั้น

ต้องทดสอบ:

- Duplicate delivery
- Consumer crash before commit
- Consumer crash after commit before ACK
- Message redelivery
- DLQ replay

---

# 25. Database Tables

```text
tenants
tenant_settings
merchants
applications
api_clients
payment_providers
provider_merchants
provider_transactions
provider_callbacks
provider_capabilities
provider_credentials_metadata
users
roles
permissions
role_permissions
user_roles
payment_transactions
payment_attempts
payment_status_history
idempotency_records
qr_payment_sessions
card_payment_sessions
refunds
void_transactions
webhook_endpoints
webhook_events
webhook_deliveries
reconciliation_batches
reconciliation_items
settlement_batches
settlement_items
approval_requests
approval_actions
audit_events
outbox_events
processed_messages
notifications
report_jobs
```

Financial record ห้าม Hard Delete

---

# 26. Partitioning Policy

ห้าม Partition ทุกตารางโดยอัตโนมัติ

High-volume candidates:

```text
audit_events
provider_callbacks
provider_transactions
payment_status_history
webhook_deliveries
```

ก่อน Partition ต้องวัด rows/day, index size, retention, query pattern และ vacuum cost

---

# 27. Audit Hash Chain

ห้ามใช้ Global Hash Chain เดียวที่ serialize write ทั้งระบบ

ใช้ Chain Scope เช่น:

```text
Tenant + Date Partition
```

ต้องมี:

- previous_hash
- event_hash
- chain_scope
- sequence
- integrity verification job
- tamper alert

---

# 28. API

Base:

```text
/api/v1
```

Payment:

```text
POST /api/v1/payments
GET  /api/v1/payments
GET  /api/v1/payments/{payment_id}
POST /api/v1/payments/{payment_id}/inquiry
POST /api/v1/payments/{payment_id}/cancel
POST /api/v1/payments/{payment_id}/refunds
POST /api/v1/payments/{payment_id}/void
```

Provider Admin:

```text
GET  /api/v1/admin/providers
GET  /api/v1/admin/providers/{provider_id}
GET  /api/v1/admin/providers/{provider_id}/health
GET  /api/v1/admin/providers/{provider_id}/capabilities
POST /api/v1/admin/providers/{provider_id}/test-connection
POST /api/v1/admin/providers/{provider_id}/maintenance-mode
```

Provider Callback:

```text
POST /api/v1/provider-callbacks/kku-payment
```

OpenAPI 3.1 ต้องเป็นส่วนหนึ่งของ CI validation

---

# 29. SQLx Offline Build — Required

Production CI/Docker Build ห้ามต้องเชื่อม Production Database เพื่อ compile query

Development/CI:

```bash
cargo sqlx prepare --workspace -- --all-targets --all-features
```

Commit:

```text
.sqlx/
```

CI:

```bash
cargo sqlx prepare --check --workspace
```

Production Build:

```bash
SQLX_OFFLINE=true cargo build --release
```

Schema/Migration change ต้อง update SQLx metadata

---

# 30. Docker Compose Production

Services:

```text
reverse-proxy
frontend
payment-api
payment-worker
kku-payment-worker
callback-worker
webhook-worker
refund-worker
reconciliation-worker
audit-worker
outbox-publisher
scheduler
postgres
redis
rabbitmq
prometheus
grafana
loki
```

Networks:

```text
frontend_net
backend_net
data_net
monitoring_net
```

Rules:

- PostgreSQL/Redis ไม่ expose internet
- RabbitMQ AMQP จำกัด backend network
- RabbitMQ Management จำกัด admin network
- Grafana/Prometheus จำกัด operations/admin
- Frontend ห้ามเข้าถึง database โดยตรง

---

# 31. Docker Compose Scaling

Scalable services:

```text
payment-api
payment-worker
kku-payment-worker
callback-worker
webhook-worker
refund-worker
audit-worker
```

ห้ามกำหนด `container_name` ให้ scalable services

เพื่อรองรับ:

```bash
docker compose up -d --scale payment-worker=4
```

Scheduler ต้องมี instance เดียว หรือใช้ job/distributed locking หาก scale ในอนาคต

---

# 32. Docker Container Security

สำหรับ Rust API/Workers:

```yaml
read_only: true
security_opt:
  - no-new-privileges:true
cap_drop:
  - ALL
tmpfs:
  - /tmp
init: true
```

และ:

- Run as non-root
- ไม่ใช้ privileged
- ไม่ mount Docker socket
- ไม่ใช้ host networking
- จำกัด bind mounts
- Runtime image minimal
- Compiler/build tools ไม่อยู่ใน runtime image
- ใช้ immutable version tag
- Production ควร pin image digest เมื่อ release แล้ว

---

# 33. Docker Secrets — No Vault

ไม่ใช้ Vault

ใช้:

- Docker Compose secrets
- Read-only mounted secret files
- Protected external environment file สำหรับ non-secret config ที่เหมาะสม

Production secret ควรอยู่นอก Git repository เช่น:

```text
/etc/cpkku-payment/secrets/
├── postgres_password
├── redis_password
├── rabbitmq_password
├── kku_client_id
├── kku_client_secret
└── webhook_master_key
```

ห้าม:

- Commit actual `.env`
- Commit secret files
- Embed secret in image
- Log secret
- Return secret in API error

Docker Compose secret คือ file-based injection ไม่ใช่ centralized secret manager

---

# 34. Docker Healthcheck Semantics

Healthcheck ใช้เพื่อ:

```text
healthy/unhealthy visibility
monitoring
dependency readiness
operations alert
```

อย่าถือว่า healthcheck failure = automatic restart

`restart: unless-stopped` ทำงานเมื่อ process/container exit ตาม restart policy

Health endpoints:

```text
/health/live
/health/ready
```

Liveness = process ยังทำงานได้

Readiness = พร้อมรับงานหรือไม่

Dependency failure อาจทำ readiness fail โดยไม่จำเป็นต้อง kill process

---

# 35. Observability

ใช้:

- tracing
- OpenTelemetry
- Prometheus
- Grafana
- Loki/OpenSearch

ทุก Request/Event มี:

```text
request_id
correlation_id
trace_id
tenant_id
application_id
payment_id
provider_code
provider_reference
```

Metrics หลัก:

```text
provider_requests_total
provider_request_duration
provider_errors_total
provider_timeouts_total
provider_callbacks_total
invalid_callbacks_total
unknown_provider_status_total
payment_created_total
payment_paid_total
payment_failed_total
rabbitmq_queue_depth
dlq_total
outbox_pending
webhook_failure_total
reconciliation_mismatch_total
```

---

# 36. Security Logging

ห้าม Log:

```text
Access Token
Refresh Token
API Secret
Password
Signing Key
Private Key
Full PAN
CVV
PIN
Track Data
Sensitive PII
Unmasked Sensitive Callback Payload
```

Logs ต้อง structured และ mask ก่อน write

---

# 37. Backup / Restore

แม้ไม่ใช้ HA/DR ต้องมี Operational Backup

PostgreSQL:

- Scheduled backup
- Encrypted backup
- Retention policy
- WAL archive/PITR หากองค์กรต้องการ recovery granularity
- Restore test

RabbitMQ:

- Definitions backup
- Persistent volume backup ตาม procedure
- Queue data ไม่ถือเป็น long-term system of record

Configuration:

- Nginx config
- Monitoring config
- Compose files
- non-secret application config

ต้องมี:

```text
scripts/backup.sh
scripts/restore.sh
```

ไม่ต้องทำ Active-Active, Active-Passive, Automatic Failover, DR Site หรือ HA Cluster

---

# 38. CI/CD Rust Quality Gate

ต้องรันอย่างน้อย:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo audit
cargo deny check
cargo sqlx prepare --check --workspace
```

Pipeline เพิ่ม:

- OpenAPI validation
- Migration validation
- Integration tests
- Provider contract tests
- E2E tests
- Secret scan
- Container scan
- SAST
- SBOM generation
- Image build
- Smoke test
- Rollback test

---

# 39. Software Supply Chain

Required:

- Commit Cargo.lock
- Dependency update ผ่าน controlled PR
- Review dependency provenance
- cargo audit
- cargo deny
- SBOM generation
- Container image scanning
- Base image version pinning
- Production image digest pinningเมื่อ release
- No unreviewed Git dependencies
- No wildcard crate versions

Known Critical/High dependency vulnerability = Release Gate Fail เว้นแต่ Security Team มี documented exception

---

# 40. Unsafe Rust Policy

First-party crates ควรใช้:

```rust
#![forbid(unsafe_code)]
```

หากจำเป็นต้องใช้ `unsafe` ต้องมี Architecture Review, Security Review, justification และ focused tests

---

# 41. Reconciliation

ใช้ 3-way reconciliation:

```text
Tenant Application
vs
CPKKU Payment
vs
KKU Payment
```

Mismatch:

```text
MATCHED
AMOUNT_MISMATCH
STATUS_MISMATCH
MISSING_IN_CPKKU
MISSING_IN_KKU_PAYMENT
DUPLICATE
UNKNOWN_PROVIDER_STATUS
MANUAL_REVIEW
RESOLVED
```

Manual Resolution ต้องมี Reason, Evidence, Approver, Audit และ Correlation ID

---

# 42. Refund / Void

Rules:

- Payment ต้องอยู่ใน state ที่อนุญาต
- Refund total ต้องไม่เกิน refundable balance
- ใช้ PostgreSQL row lock/transaction
- Maker ห้าม approve รายการตนเอง
- Capability ต้องยืนยันจาก KKU Payment
- Timeout ใช้ inquiry-before-retry
- ทุก operation มี audit

---

# 43. Tenant Webhook

CPKKU -> Tenant webhook ต้องรองรับ:

- Event ID
- Event Version
- Tenant ID
- Correlation ID
- Timestamp
- HMAC signature
- Retry
- DLQ
- Delivery History
- Replay
- Secret Rotation

Replay ต้องสร้าง attempt ใหม่และมี Audit Log

---

# 44. Testing

Unit:

- Domain
- State Machine
- Idempotency
- Provider Registry
- Status Mapping
- Error Mapping
- Validation
- Callback Signature

Integration:

- PostgreSQL/SQLx
- Redis
- RabbitMQ
- Mock KKU Payment
- Outbox
- Inbox

Contract:

- Create Payment Success/Pending/Timeout
- Auth/Permission Failure
- Inquiry
- Callback Success/Duplicate/Invalid Signature
- Refund/Void
- Unknown Status
- Rate Limit
- Provider Unavailable

Security:

- Cross-Tenant
- RLS Connection Reuse
- Replay
- SQL Injection
- XSS/CSRF/SSRF
- BOLA/BFLA
- Secret Exposure

Reliability:

- Duplicate Payment
- Duplicate Message
- Callback redelivery
- Worker crash
- RabbitMQ outage
- Redis outage
- Provider timeout/5xx
- Outbox recovery

Performance:

- Create Payment
- Inquiry
- Callback burst
- Webhook burst
- Dashboard
- Reconciliation batch
- Multi-Tenant concurrency

---

# 45. Development Phases

```text
Phase 0  Discovery + KKU Payment Contract
Phase 1  Rust Workspace Foundation
Phase 2  Provider Abstraction
Phase 3  KKU Payment Adapter
Phase 4  Multi-Tenant + Identity + RLS
Phase 5  Payment Core + PostgreSQL Idempotency
Phase 6  QR Payment
Phase 7  Credit Payment
Phase 8  RabbitMQ + Outbox/Inbox
Phase 9  Callback + Timeout Recovery
Phase 10 Refund + Void
Phase 11 Tenant Webhook
Phase 12 Reconciliation
Phase 13 Settlement + Reporting
Phase 14 Dashboard + Operations
Phase 15 Audit + Security + Compliance
Phase 16 Docker Compose Production Hardening
Phase 17 CI/CD + Production Gate
```

---

# 46. Production Deployment

ห้ามใช้ image tag `latest`

ใช้ versioned tags เช่น:

```text
cpkku-payment-api:1.0.0
cpkku-payment-worker:1.0.0
```

Deploy:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml pull
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
docker compose -f docker-compose.yml -f docker-compose.prod.yml ps
```

Deploy เฉพาะ service:

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --no-deps payment-api
```

ต้องมี rollback script ไปยัง version ก่อนหน้า

---

# 47. Production Acceptance Criteria

1. Rust 1.98.1 build ผ่าน
2. Cargo.lock ถูก commit
3. SQLx offline check ผ่าน
4. Multi-Tenant Isolation ผ่าน
5. PostgreSQL RLS pooled-connection test ผ่าน
6. PostgreSQL idempotency ผ่าน
7. Redis restart แล้ว duplicate protection ยังผ่าน
8. QR Payment ผ่าน
9. Credit Payment ผ่านตาม Provider Contract
10. Provider Timeout ไม่สร้าง duplicate
11. Inquiry-before-retry ผ่าน
12. Callback raw-body signature verification ผ่าน
13. Callback persist-before-business-processing ผ่าน
14. Duplicate Callback ผ่าน
15. RabbitMQ Outbox/Inbox ผ่าน
16. Worker crash/redelivery test ผ่าน
17. Retry/DLQ ผ่าน
18. Refund/Void consistency ผ่าน
19. Reconciliation ผ่าน
20. Settlement ผ่าน
21. Audit integrity verification ผ่าน
22. Monitoring/Alert พร้อม
23. Docker Compose security baseline ผ่าน
24. Backup/Restore ผ่าน
25. Provider Contract Test ผ่าน
26. Security Scan ผ่าน
27. Penetration Test ไม่มี unresolved Critical/High
28. Load Test ผ่าน SLA
29. Rollback Test ผ่าน
30. UAT/Finance/Security/Operations Sign-off ครบ

---

# 48. คำสั่งเริ่มต้นสำหรับ Codex

```text
Project: CPKKU Payment
Backend: Rust 1.98.1 Stable / Rust 2024
Deployment: Docker Compose Only
Provider: KKU Payment

Provider Manual:
https://unipay.kku.ac.th/docs/v1/kku#description/introduction

Start with Phase 0 and Phase 1.

Tasks:
1. Inspect the existing repository.
2. Show the repository tree.
3. Locate all old Go/Gin/sqlc dependencies if present.
4. Produce a Go-to-Rust migration/refactor report if legacy Go code exists.
5. Read the official KKU Payment Provider Manual.
6. Locate/export OpenAPI JSON/YAML if available.
7. Create Provider Contract Mapping.
8. List verified endpoints, methods, authentication, headers, request/response schemas, callback behavior, statuses and errors.
9. List every unverified contract item as an Open Question.
10. Create a Rust 1.98.1 Cargo Workspace.
11. Use Rust 2024 Edition.
12. Add Axum, Tokio, Tower, SQLx, Lapin, Reqwest/Rustls, Serde, rust_decimal, thiserror, anyhow, tracing, OpenTelemetry, utoipa, validator and async-trait as required.
13. Implement Domain/Application/Infrastructure boundaries.
14. Implement Provider Abstraction and Mock Provider.
15. Create KKU Payment Adapter skeleton only for unverified operations.
16. Implement PostgreSQL transaction-local tenant context for RLS.
17. Implement PostgreSQL-backed financial idempotency.
18. Configure SQLx offline metadata workflow.
19. Configure RabbitMQ single-node durable messaging.
20. Implement Transactional Outbox and Inbox skeletons.
21. Configure raw-body-safe provider callback handling.
22. Create Docker Compose architecture.
23. Do not set container_name on scalable app/worker services.
24. Apply non-root, cap_drop, no-new-privileges, read_only and tmpfs controls where compatible.
25. Configure Prometheus/Grafana/logging/OpenTelemetry.
26. Add cargo fmt/clippy/test/audit/deny/sqlx checks to CI.
27. Generate SBOM and scan container image.
28. Provide tests and documentation.
29. Do not implement any KKU Payment contract that is not verified.
30. Stop at documented TODO/Open Question rather than inventing provider behavior.

Every response must include:
1. Summary
2. Current Phase
3. Files Created
4. Files Modified
5. Architecture Decisions
6. Provider Manual Mapping
7. Database Changes
8. RLS / Tenant Isolation Changes
9. Idempotency Changes
10. API Changes
11. RabbitMQ Changes
12. Security Changes
13. Tests Added
14. CI/CD Changes
15. How to Run
16. Known Limitations
17. Open Questions
18. Next Step
```

---

# 49. Architecture Decisions — Final Baseline

```text
Frontend
    Next.js + TypeScript

Backend
    Rust 1.98.1 Stable
    Rust 2024
    Axum + Tokio + Tower
    SQLx
    Lapin
    Reqwest + Rustls
    Serde
    rust_decimal
    tracing + OpenTelemetry

Source of Truth
    PostgreSQL

Idempotency
    PostgreSQL
    Redis only optional optimization

Tenant Isolation
    Application scope + PostgreSQL RLS
    transaction-local tenant context

Messaging
    RabbitMQ Single Node
    durable queues
    persistent messages
    publisher confirm
    manual ACK
    transactional outbox
    inbox/idempotent consumer
    retry/DLQ

Provider
    KKU Payment
    Contract-first adapter

Create Payment
    Synchronous provider call
    async recovery/inquiry

Callback
    exact raw bytes
    verify contract-required security
    persist first
    ACK quickly
    async business processing

Deployment
    Docker Compose Only

Secrets
    Docker Compose secrets / protected mounted files outside Git

Observability
    Prometheus
    Grafana
    Loki/OpenSearch
    OpenTelemetry

Operations
    Backup / Restore
    No HA
    No DR Site

Security
    Non-root containers
    no-new-privileges
    cap_drop ALL
    read-only filesystem where compatible
    supply-chain scanning
    SBOM
    immutable production images
```

---

# 50. Definition of Done

แต่ละ Phase ต้องผ่าน:

- Compile
- cargo fmt --check
- cargo clippy -D warnings
- Unit Test
- Integration Test ที่เกี่ยวข้อง
- SQLx metadata check
- Contract Test ที่เกี่ยวข้อง
- Security Scan
- No secret leakage
- Documentation Update
- OpenAPI Update
- Migration Test
- Audit coverage
- Monitoring coverage
- Docker Compose test
- Acceptance Criteria

ห้ามถือว่างานเสร็จหากยังมี unresolved Critical defect หรือมี Provider Contract ที่ถูกเดาขึ้นเอง

---

# 51. KKU ERP Manual Receipt Integration

## 51.1 Architecture Boundary

ฟังก์ชันออกใบเสร็จเป็น External Integration แยกจาก KKU Payment Provider

```text
                           +-------------------+
                           |   KKU Payment     |
                           | Payment Provider  |
                           +---------+---------+
                                     |
                                     v
Tenant -> CPKKU Payment -> Payment Core
                               |
                               | Payment = PAID
                               v
                         Receipt Service
                               |
                 +-------------+--------------+
                 |                            |
                 v                            v
       Oracle Identity / OAuth2       KKU ERP Receipt Adapter
                 |                            |
                 +------------+---------------+
                              |
                              v
                   Oracle Integration Cloud
                     CreateMiscReceipt
                              |
                              v
                        Oracle Fusion ERP
                              |
                              v
                       Receipt Metadata
                              |
                              v
                 KKU Receipt PDF File Service
                     interface.kku.ac.th
```

ห้ามรวม ERP Receipt API เข้า `KKU Payment Adapter`

ให้สร้าง Integration แยก เช่น:

```text
crates/kku-erp-receipt/
```

---

## 51.2 Source Manual

ใช้เอกสารแนบ:

```text
API Manaulมออกใบเสร็จ-KKU.docx
```

เป็น Source of Truth สำหรับ ERP Receipt Integration

ข้อมูลที่ยืนยันจากเอกสาร:

### ERP Token

```text
Method:
POST

Endpoint:
https://idcs-148247ef10d44f5d85a73f179a12aacd.identity.oraclecloud.com:443/oauth2/v1/token

Content-Type:
application/x-www-form-urlencoded

grant_type:
client_credentials
```

ต้องส่ง:

```text
client_id
client_secret
scope
```

ค่า `scope` ต้องเก็บเป็น Configuration/Secret-backed setting และต้องตรวจสอบค่าจริงจากเอกสาร/ERP team ก่อน Production เพราะข้อความในเอกสารมีการตัดบรรทัดของ URI

Response:

```json
{
  "access_token": "...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

Access Token:

- ห้าม Persist ใน PostgreSQL
- ห้าม Log
- Cache ใน memory ของ receipt worker/service
- Refresh ก่อนหมดอายุ
- ใช้ monotonic expiry safety margin
- Token request สามารถ retry แบบ controlled retry ได้

---

## 51.3 Payment Verification Before Receipt

ก่อนออกใบเสร็จต้องตรวจสอบว่ารายการชำระเงินจริงสำเร็จ

เอกสารเดิมกล่าวถึง:

```text
KkuPay::kkupaid($cenroll_id)
```

แต่ระบบใหม่เป็น Rust จึงห้าม dependency กับ PHP function นี้

ให้สร้าง:

```rust
pub trait PaymentVerificationService {
    async fn verify_paid(
        &self,
        payment_id: PaymentId,
    ) -> Result<VerifiedPaidTransaction, PaymentVerificationError>;
}
```

Service ต้องตรวจสอบจาก CPKKU Payment Source of Truth และ KKU Payment Provider ตาม policy

Receipt ห้ามสร้างหาก:

```text
payment.status != PAID
```

ค่า Provider Reference ที่ใช้กับใบเสร็จต้องมาจาก transaction ที่ยืนยันแล้ว

---

## 51.4 Create Misc Receipt API

เอกสารระบุ SIT/UAT endpoint:

```text
Method:
POST

Endpoint:
https://sit-uat-oic797-axacomqa0kjr-si.integration.ap-singapore-1.ocp.oraclecloud.com/ic/api/integration/v1/flows/rest/KKU_INT_FIN_AR_05_MISC_RCPT_APP/1.0/CreateMiscReceipt

Content-Type:
application/json

Authorization:
Bearer <ERP access token>
```

สำคัญ:

```text
The endpoint above is SIT/UAT.
Do NOT use it as a Production endpoint unless explicitly confirmed.
Production ERP endpoint MUST be provided/configured separately.
```

Configuration:

```text
KKU_ERP_RECEIPT_BASE_URL
KKU_ERP_RECEIPT_CREATE_PATH
KKU_ERP_OAUTH_TOKEN_URL
KKU_ERP_OAUTH_CLIENT_ID_FILE
KKU_ERP_OAUTH_CLIENT_SECRET_FILE
KKU_ERP_OAUTH_SCOPE
```

---

## 51.5 Receipt Request Fields

Request Contract:

```json
{
  "ReceiptData": [
    {
      "ReceiptIssuedBy": "...",
      "ReceiptDescription": "...",
      "Ref3": "...",
      "ProjectId": "...",
      "TransactionId": "...",
      "ReceiptAmount": "...",
      "CustomerPhone": "...",
      "CustomerEmail": "...",
      "Ref2": "...",
      "CustomerName": "...",
      "ReceiptDate": "YYYY-MM-DD",
      "CustomerTaxId": "...",
      "CustomerAddress": "...",
      "Service": "...",
      "Year": "...",
      "BankAccount": "..."
    }
  ]
}
```

### Mapping

| ERP Field          | Source                                                       |
| ------------------ | ------------------------------------------------------------ |
| ReceiptIssuedBy    | Config/default `งานการเงิน(CPKKU)`                           |
| ReceiptDescription | Receipt context / course description                         |
| Ref3               | KKU Payment verified transaction                             |
| ProjectId          | Config/default `CPKKUPAYMENT`                                |
| TransactionId      | KKU Payment `transaction_id`                                 |
| ReceiptAmount      | Authoritative amount / legacy `course.course_price_std`      |
| CustomerPhone      | Customer snapshot / legacy `user.phone`                      |
| CustomerEmail      | Customer snapshot / legacy `user.email`                      |
| Ref2               | KKU Payment verified transaction                             |
| CustomerName       | Customer fullname snapshot                                   |
| ReceiptDate        | Date from KKU Payment `bank_ref_time`                        |
| CustomerTaxId      | Customer tax/id-card snapshot                                |
| CustomerAddress    | Config/default `-` unless authoritative address is available |
| Service            | Config/default `0201`                                        |
| Year               | Receipt context / legacy `course.course_year`                |
| BankAccount        | Config/default `SCB-CA-5513030421`                           |

ข้อกำหนด:

- `TransactionId`, `Ref2`, `Ref3`, `ReceiptDate` ห้ามรับค่าจากผู้ใช้งาน Manual API โดยตรง
- `ReceiptAmount` ต้องมาจาก authoritative business/payment data
- ห้ามให้ Operator override amount
- `TransactionId` ต้องเก็บเป็น String/VARCHAR ไม่ใช่ JavaScript Number
- `CustomerTaxId` เป็น Sensitive PII ต้อง mask ใน log/audit
- Receipt request ต้องเก็บ snapshot ของข้อมูลที่ใช้สร้างใบเสร็จเพื่อ Audit

---

## 51.6 Decouple Receipt From Legacy Course/User Tables

เอกสารเดิมอ้างถึง:

```text
course.course_price_std
course.course_year
user.phone
user.email
user.student_id_card
User fullname
```

CPKKU Payment Core ไม่ควร query legacy application database โดยตรง

ให้ใช้ `ReceiptContextResolver`:

```rust
#[async_trait::async_trait]
pub trait ReceiptContextResolver: Send + Sync {
    async fn resolve(
        &self,
        payment_id: PaymentId,
    ) -> Result<ReceiptContext, ReceiptContextError>;
}
```

แนวทางที่แนะนำ:

1. Capture receipt/customer snapshot ตั้งแต่ Create Payment
2. หรือใช้ tenant/application-specific integration adapter
3. ห้าม Payment Core ต่อ DB ของระบบ Course/User โดยตรง

ตัวอย่าง snapshot:

```text
receipt_description
receipt_amount
customer_name
customer_phone
customer_email
customer_tax_id
customer_address
fiscal_year
service_code
bank_account
```

---

## 51.7 Manual Receipt API

เพิ่ม API สำหรับเจ้าหน้าที่:

```text
POST /api/v1/payments/{payment_id}/receipts/manual
```

Request:

```json
{
  "reason": "ออกใบเสร็จย้อนหลังตามคำร้อง",
  "force_provider_inquiry": true
}
```

ห้ามให้ Request Body ส่ง:

```text
receipt_amount
transaction_id
ref2
ref3
receipt_date
project_id
bank_account
```

เพราะเป็น authoritative/server-controlled data

Response:

```text
HTTP 202 Accepted
```

ตัวอย่าง:

```json
{
  "data": {
    "receipt_request_id": "uuid-v7",
    "payment_id": "uuid-v7",
    "status": "QUEUED"
  }
}
```

เพิ่ม API:

```text
GET  /api/v1/payments/{payment_id}/receipts
GET  /api/v1/receipts/{receipt_id}
GET  /api/v1/receipts/{receipt_id}/pdf
POST /api/v1/receipts/{receipt_id}/retry
POST /api/v1/receipts/{receipt_id}/resolve-manual-review
```

`retry` ต้องผ่าน safe-retry policy เท่านั้น

---

## 51.8 Receipt State Machine

ใช้ Internal Receipt Status:

```text
REQUESTED
VERIFYING_PAYMENT
QUEUED
PROCESSING
ISSUED
FAILED
MANUAL_REVIEW
```

Flow:

```text
REQUESTED
   |
   v
VERIFYING_PAYMENT
   |
   +-- Not Paid ------> FAILED
   |
   v
QUEUED
   |
   v
PROCESSING
   |
   +-- Success -------> ISSUED
   |
   +-- Definitive Error -> FAILED
   |
   +-- Ambiguous Result -> MANUAL_REVIEW
```

---

## 51.9 Critical Idempotency Rule

ออกใบเสร็จเป็น Financial Mutation

ต้องมี Database uniqueness อย่างน้อย:

```text
UNIQUE(payment_id, receipt_type)
```

หรือ policy ที่รองรับ multiple legitimate receipts หาก Finance ยืนยัน

เพิ่ม:

```text
receipt_request_id
payment_id
provider_transaction_id
request_hash
erp_document_number
oracle_fusion_receipt_id
oracle_fusion_receipt_number
```

ห้ามออกใบเสร็จซ้ำเพราะผู้ใช้กดปุ่มซ้ำ

Manual API ต้องใช้ Idempotency-Key

Scope:

```text
tenant_id + payment_id + MANUAL_RECEIPT + idempotency_key
```

---

## 51.10 Unsafe Retry Protection

เอกสารที่ให้มายังไม่ระบุ Receipt Inquiry/Search API สำหรับตรวจว่ารายการถูกสร้างใน Oracle Fusion แล้วหรือไม่

ดังนั้น:

```text
POST CreateMiscReceipt
```

หากเกิด:

```text
timeout after request may have been sent
connection reset after write
ambiguous HTTP failure
```

ห้าม retry แบบอัตโนมัติ

ให้:

```text
status = MANUAL_REVIEW
```

จนกว่าจะ:

- มี ERP Receipt Inquiry API ที่ยืนยันได้
- หรือ Finance/ERP Team ตรวจสอบ Document/TransactionId
- หรือมี idempotency guarantee จาก ERP contract

สามารถ retry อัตโนมัติได้เฉพาะ failure ที่พิสูจน์ได้ว่า request ยังไม่ได้ถูกส่ง เช่น validation/local configuration error ตาม retry policy ที่ชัดเจน

---

## 51.11 ERP Receipt Response

Response ที่เอกสารระบุ:

```json
{
  "result": [
    {
      "OracleFusionReceiptNumber": "OTHER_01072025071256040",
      "OracleFusionReceiptId": "74023",
      "StatusCode": "SUCCESS",
      "StatusMessage": "Receipt created successfully in Oracle Fusion",
      "DocumentNumber": "68290900000002",
      "PDFFileName": "68290900000002.pdf"
    }
  ]
}
```

เก็บข้อมูล:

```text
oracle_fusion_receipt_number
oracle_fusion_receipt_id
status_code
status_message
document_number
pdf_file_name
issued_at
```

อย่าใช้ `StatusCode == SUCCESS` โดยเดา case/possible values เพิ่มเติม
ให้ mapping ตาม ERP contract เท่านั้น

---

## 51.12 Receipt PDF Download

เอกสารระบุ:

```text
Method:
GET

Base URL:
https://interface.kku.ac.th/kkuutils2/files/

Authentication:
Basic Auth
```

URL:

```text
https://interface.kku.ac.th/kkuutils2/files/{PDFFileName}
```

เพิ่ม CPKKU API:

```text
GET /api/v1/receipts/{receipt_id}/pdf
```

Backend ต้องเป็นผู้เรียก KKU file service

ห้ามส่ง Basic Auth credential ไป Browser

Secret:

```text
KKU_RECEIPT_FILE_USERNAME_FILE
KKU_RECEIPT_FILE_PASSWORD_FILE
KKU_RECEIPT_FILE_BASE_URL
```

Security:

- Validate `PDFFileName`
- Reject path traversal เช่น `../`
- Allow only expected `.pdf` filename pattern
- URL encode filename
- Verify HTTP Content-Type
- Enforce maximum file size
- Set timeout
- Stream response
- Audit every download
- Tenant authorization ก่อน download

---

## 51.13 Receipt Database Tables

เพิ่ม:

```text
receipt_profiles
receipt_requests
receipts
receipt_attempts
receipt_documents
```

### receipt_profiles

เก็บ configuration ระดับ Tenant/Application:

```text
id
tenant_id
application_id
receipt_issued_by
project_id
service_code
bank_account
erp_environment
active
created_at
updated_at
```

ค่าตั้งต้นจากคู่มือ:

```text
receipt_issued_by = งานการเงิน(CPKKU)
project_id        = CPKKUPAYMENT
service_code      = 0201
bank_account      = SCB-CA-5513030421
```

### receipt_requests

```text
id
tenant_id
payment_id
requested_by
request_reason
idempotency_key
request_hash
status
created_at
updated_at
```

### receipts

```text
id
tenant_id
payment_id
receipt_request_id

transaction_id
ref2
ref3
receipt_amount
receipt_date

receipt_issued_by
receipt_description
project_id
service_code
fiscal_year
bank_account

customer_name
customer_phone
customer_email
customer_tax_id_encrypted_or_protected
customer_address

oracle_fusion_receipt_number
oracle_fusion_receipt_id
document_number
pdf_file_name

status
issued_at
created_at
updated_at
```

### receipt_attempts

```text
id
receipt_id
attempt_no
request_payload_masked
response_payload_masked
http_status
result
error_code
error_message_masked
started_at
completed_at
```

---

## 51.14 Rust Module Structure

```text
crates/domain/src/receipt/
├── entity.rs
├── value_objects.rs
├── status.rs
├── repository.rs
└── errors.rs

crates/application/src/receipt/
├── issue_manual.rs
├── verify_payment.rs
├── get_receipt.rs
├── download_pdf.rs
├── retry.rs
└── context_resolver.rs

crates/kku-erp-receipt/src/
├── client.rs
├── auth.rs
├── token_manager.rs
├── create_receipt.rs
├── response_mapper.rs
├── pdf_client.rs
├── config.rs
├── models.rs
└── errors.rs

apps/receipt-worker/
```

เพิ่ม Docker Compose service:

```text
receipt-worker
```

---

## 51.15 RabbitMQ Events

เพิ่ม routing keys:

```text
receipt.manual.requested
receipt.payment.verified
receipt.issue.requested
receipt.issue.processing
receipt.issued
receipt.issue.failed
receipt.manual_review.required
receipt.pdf.downloaded
```

Queue:

```text
q.receipt.issue
q.receipt.manual-review
```

`CreateMiscReceipt` เป็น external financial mutation:

- ใช้ Inbox
- ใช้ Database idempotency
- ห้าม blind retry เมื่อ outcome ambiguous

---

## 51.16 RBAC

เพิ่ม permissions:

```text
receipts.read
receipts.manual.create
receipts.download
receipts.retry
receipts.manual_review.resolve
receipts.audit.read
```

แนะนำ:

- Tenant Finance / Payment Operations สามารถ manual create ตาม scope
- Retry/Manual Review Resolution เป็นสิทธิ์ระดับสูงกว่า
- ทุก Manual Receipt ต้องมี `reason`
- Maker/Checker เปิดใช้ได้ตาม tenant/finance policy

---

## 51.17 Audit Events

เพิ่ม:

```text
RECEIPT_MANUAL_REQUESTED
RECEIPT_PAYMENT_VERIFIED
RECEIPT_ERP_TOKEN_ACQUIRED
RECEIPT_CREATE_SUBMITTED
RECEIPT_CREATE_SUCCEEDED
RECEIPT_CREATE_FAILED
RECEIPT_MANUAL_REVIEW_REQUIRED
RECEIPT_RETRY_REQUESTED
RECEIPT_PDF_DOWNLOADED
```

ห้าม Audit:

```text
access_token
client_secret
basic_auth_password
full customer tax id
unmasked sensitive payload
```

---

## 51.18 Docker Compose Secrets

เพิ่ม external secret files:

```text
/etc/cpkku-payment/secrets/
├── kku_erp_oauth_client_id
├── kku_erp_oauth_client_secret
├── kku_receipt_file_username
└── kku_receipt_file_password
```

ห้ามเก็บใน Git repository

---

## 51.19 Observability

Metrics:

```text
receipt_manual_requests_total
receipt_issue_success_total
receipt_issue_failed_total
receipt_manual_review_total
receipt_issue_duration_seconds
receipt_erp_token_errors_total
receipt_pdf_download_total
receipt_pdf_download_errors_total
```

Alert:

```text
ERP token acquisition failures
CreateMiscReceipt failure surge
Manual review backlog
PDF download failure surge
Receipt processing backlog
```

---

## 51.20 Tests

ต้องมี:

### Unit

- Receipt field mapper
- Receipt state machine
- Receipt idempotency
- Payment-paid precondition
- TransactionId string handling
- Receipt date mapping from bank_ref_time
- PII masking
- PDF filename validation

### Integration

- OAuth token mock
- OIC CreateMiscReceipt mock
- Basic Auth PDF mock
- PostgreSQL
- RabbitMQ
- receipt-worker

### Security

- User without permission
- Cross-tenant receipt read/download
- CustomerTaxId log masking
- Path traversal in PDF filename
- Oversized PDF
- Invalid Content-Type
- Secret leakage

### Reliability

- Duplicate manual request
- Duplicate RabbitMQ message
- Worker crash
- Token expiration
- ERP timeout before send
- Ambiguous timeout after send
- PDF service unavailable

---

## 51.21 Receipt Acceptance Criteria

ฟังก์ชัน Manual Receipt พร้อม Production เมื่อ:

1. ออกใบเสร็จได้เฉพาะ Payment = PAID
2. TransactionId/Ref2/Ref3 มาจาก verified KKU Payment transaction
3. ReceiptDate มาจาก `bank_ref_time`
4. ReceiptAmount มาจาก authoritative data
5. Manual caller override financial fields ไม่ได้
6. OAuth2 ERP Token ทำงานและไม่รั่วใน log
7. CreateMiscReceipt mapping ตรงกับ ERP manual
8. Duplicate manual requests ไม่สร้างใบเสร็จซ้ำ
9. Ambiguous external timeout ไม่ blind retry
10. Oracle receipt metadata ถูกบันทึกครบ
11. PDF download ผ่าน Backend และไม่ expose Basic Auth
12. PDF filename path traversal test ผ่าน
13. Tenant/RBAC isolation ผ่าน
14. CustomerTaxId/PII masking ผ่าน
15. Audit events ครบ
16. Monitoring/alerts พร้อม
17. Docker Compose `receipt-worker` deploy ผ่าน
18. SIT/UAT integration test ผ่าน
19. Production ERP endpoint/credential ได้รับการยืนยันก่อน Go-Live

---

# 52. Revised Development Phases

ปรับ Phase Plan:

```text
Phase 0  Discovery + KKU Payment Contract + ERP Receipt Contract
Phase 1  Rust Workspace Foundation
Phase 2  Provider Abstraction
Phase 3  KKU Payment Adapter
Phase 4  Multi-Tenant + Identity + RLS
Phase 5  Payment Core + PostgreSQL Idempotency
Phase 6  QR Payment
Phase 7  Credit Payment
Phase 8  RabbitMQ + Outbox/Inbox
Phase 9  Callback + Timeout Recovery
Phase 10 Refund + Void
Phase 11 Tenant Webhook
Phase 12 Manual Receipt + KKU ERP Integration
Phase 13 Reconciliation
Phase 14 Settlement + Reporting
Phase 15 Dashboard + Operations
Phase 16 Audit + Security + Compliance
Phase 17 Docker Compose Production Hardening
Phase 18 CI/CD + Production Gate
```

---

# 53. Additional Codex Instruction — Manual Receipt

```text
Implement the CPKKU Manual Receipt module as a separate ERP integration.

Source manual:
API Manaulมออกใบเสร็จ-KKU.docx

Do NOT place ERP receipt logic inside the KKU Payment provider adapter.

Architecture:

CPKKU Payment
  -> Receipt Service
  -> KKU ERP Receipt Adapter
  -> Oracle Identity OAuth2
  -> Oracle Integration CreateMiscReceipt
  -> Oracle Fusion ERP

Receipt PDF:
CPKKU Receipt API
  -> KKU Receipt File Service
  -> interface.kku.ac.th

Before issuing a receipt:
1. Verify payment is PAID.
2. Resolve authoritative KKU Payment TransactionId, Ref2, Ref3.
3. Use bank_ref_time as ReceiptDate.
4. Resolve authoritative receipt/customer snapshot.
5. Enforce PostgreSQL idempotency.
6. Do not permit manual amount/reference override.
7. Obtain ERP OAuth token.
8. Call CreateMiscReceipt.
9. Persist Oracle receipt metadata.
10. Treat ambiguous mutation timeout as MANUAL_REVIEW.
11. Do not blind retry CreateMiscReceipt.
12. Download PDF server-side using Basic Auth; never expose credentials to browser.
13. Add RBAC, audit, metrics, integration tests and security tests.
14. Keep SIT/UAT endpoint configurable and do not assume it is the Production endpoint.
```
