# Contributing to CP-KKU Payment Gateway

This is the team's dev workflow — for humans and AI coding agents alike. If
you (or an agent working on your behalf) are picking up work here, this file
is the source of truth for how work moves from idea to merged code.

## Dev Flow

```text
Grooming
  (break requirement docs in docs/ into scoped tasks)
        |
        v
Create GitLab Issue
  (one task = one issue; acceptance criteria written into the issue body)
        |
        v
Dev
  (branch off main, implement against the issue's acceptance criteria)
        |
        v
Commit
        |
        v
Open Merge Request
  (references the issue; CI must be green)
        |
        v
Review
  (reviewer checks the MR against the issue's acceptance criteria)
        |
    +---+---+------------------+
    |       |                  |
 Approve  Comment       Request Changes
    |       |                  |
    |       +--- discussion, no gate ---+
    |                                    |
    v                                    v
 Merge                              Fix code
    |                                    |
    v                                    |
Issue auto-closes  <--------------------+
  (via "Closes #N" in the MR description)
```

## Branching and Commits

- Branch name includes the issue number: `feat/123-payment-core`,
  `fix/456-idempotency-race`.
- Commit messages describe *why*, not just *what* — see existing history for
  style. Reference the issue number where it adds context.
- Never force-push a branch someone else is reviewing without saying so first.

## Merge Requests

- One MR = one issue. If an MR grows to cover unrelated work, split it.
- MR description must include `Closes #<issue-number>` so merging closes the
  issue automatically.
- MR description restates the issue's acceptance criteria as a checklist —
  the reviewer checks off against this, not against vibes.

### Definition of Done checklist

Every MR must satisfy this before it's ready for review — this mirrors
`docs/CPKKU_Payment_Master_Development_Prompt_v3_Rust_DockerCompose_Receipt.md`,
section 50 exactly, so that document stays the single source of truth. Mark
an item N/A in the PR description if the change genuinely doesn't touch it —
don't silently skip it.

**Automated (run locally before pushing; CI re-checks all of these):**

- [ ] Compiles (`cargo build --workspace`)
- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] Unit tests pass
- [ ] Relevant integration tests pass
- [ ] `cargo sqlx prepare --check --workspace` (once migrations exist)
- [ ] Relevant provider contract tests pass
- [ ] Migration test (if the PR adds/changes a migration)
- [ ] Docker Compose test (if the MR touches a service's runtime behavior)

**Manual (author confirms in the MR description):**

- [ ] Security scan reviewed, nothing new flagged
- [ ] No secret leakage (checked diff for tokens/keys/passwords/PII in logs)
- [ ] Documentation updated (README/CONTRIBUTING/module docs, as relevant)
- [ ] OpenAPI spec updated with realistic examples (see "API Documentation & Examples" below, if the MR changes an API contract)
- [ ] Audit coverage — new state changes emit an audit event (if applicable)
- [ ] Monitoring coverage — new failure modes have a metric/alert (if applicable)
- [ ] Issue's acceptance criteria met

No unresolved Critical defect, and no guessed Provider Contract behavior, may
be merged — full stop. See "Working on an Open Question" below.

(Full CI command gate: section 38. Test category taxonomy — which tests to
write for what kind of change — section 44.)

## API Documentation & Examples

We do not maintain a separate Postman/Bruno/Insomnia collection. The OpenAPI
3.1 spec, generated from code via `utoipa` (dev prompt section 6), is the
single source of truth for both the contract and example requests/responses
— the same pattern `api-payment kku.json` (the KKU Payment provider spec)
already uses, where every field carries a realistic `example` value.

- Any endpoint you add or change in the `api` crate must have `utoipa`
  annotations with realistic `example` values on request and response
  bodies — not placeholder values like `"string"` or `0`.
- The `api` crate serves this spec through `utoipa-swagger-ui`, giving an
  in-browser "try it out" UI — this is what replaces a Postman collection.
  No exported file to keep in sync, no drift between docs and code.
- If you genuinely need a portable collection for an external audience
  (e.g. handing something to a tenant integration team before the Developer
  Portal exists), generate it from the OpenAPI spec on demand rather than
  hand-maintaining one — treat any such export as disposable.

## Review

Reviews resolve to exactly one of three states — no numeric scores, no
ambiguity about whether a review blocks merge:

| State | Meaning | Blocks merge? |
|---|---|---|
| **Approve** | Meets the issue's acceptance criteria, no blocking concerns | No |
| **Comment** | Observations, suggestions, questions — nothing that must change | No |
| **Request Changes** | At least one blocking issue against the acceptance criteria | Yes |

`Request Changes` must say *what* acceptance criterion isn't met, not just
"this feels off." The author fixes, pushes, and re-requests review. The issue
stays open until an MR that closes it is merged.

If a team wants a quality-trend metric later (e.g. a 1–10 score), track that
as separate, non-blocking metadata (e.g. a comment or a project field) —
never mix it into the merge gate above.

## Working on an Open Question

Some tasks touch the KKU Payment or KKU ERP integration, which have
documented gaps (see `README.md` → "Open questions against the KKU Payment
contract"). The dev prompt's Provider Contract Rules apply here without
exception:

- Never guess provider behavior, request/response shape, status codes, or
  callback format.
- If a contract detail isn't verified, write it into the issue (or a new
  issue) as an **Open Question** and stop — don't implement against a guess.
- Provider-generated models stay inside the adapter crate
  (`provider-kku-payment`, `kku-erp-receipt`); they must never leak into
  `domain`.

## For AI Coding Agents

If you are an AI agent picking up an issue in this repo:

1. **Follow this flow exactly** — don't skip straight to code without an
   issue, and don't merge or close your own MR. A human approves and merges.
2. **Don't guess the provider contract.** If `api-payment kku.json` or the
   ERP manual (`docs/`) doesn't cover what you need, stop and post an Open
   Question on the issue instead of inventing behavior.
3. **Match the MR description to the issue's acceptance criteria**,
   checklist-style, so the human reviewer can check off against something
   concrete instead of re-deriving what the task was.
4. **Run the CI gate locally before opening the MR** (`fmt`, `clippy`,
   `test`) — don't rely on remote CI to catch formatting/lint issues a local
   run would've caught in seconds.
5. **State assumptions explicitly** in the MR description whenever the issue
   was ambiguous and you had to pick an interpretation — the reviewer needs
   to know what to double-check.
