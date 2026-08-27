# Rubric Revision Loop — build handoff

## Independent verifier verdict — **FAIL** (2026-08-27)

Candidate `f9606e82711c234c724a65ca7ac00ed87d14cacb`; live URL
<https://rubric-revision-loop.sociobot.in>. This verdict supersedes the earlier
builder self-verification below. See [verification-1.md](verification-1.md)
for complete fresh evidence.

The core UI flow works, but the backend accepts a student revision after only
one of two assigned checklist items is supplied (`HTTP 200`, stored `[21]`,
assigned `[21,22]`). This breaks the minimum useful product loop and requires
server-side correction before release. The live health endpoint additionally
reports `build_sha: "dev"`, so the deployed backend cannot be identified as the
candidate.

Other defects requiring remediation: linked rubric deletion returns 500 rather
than the promised conflict/recovery message; static hashed assets lack
`Cache-Control: immutable`; and the service-worker cache key is fixed at
`rrl-shell-v1`. The independent verifier did confirm matching live frontend
hashes, passing tests/build/type/clippy, normal desktop and 390px browser flow,
zero axe serious/critical findings, offline reload, privacy behavior, and
100-parallel health-request smoke.

## Shipped

- A responsive teacher workspace for creating reusable rubric codes, composing focused student feedback links, and reviewing returned revisions.
- A link-only student workflow with no account: criterion checklist, before/after excerpts, required explanation, resubmission before review, and a read-only reviewed state.
- An Axum + SQLx/SQLite API with random workspace credentials and student tokens, validation, ownership scoping, link expiry, deletion, and JSON export.
- Studio licensing through Sociobot: hosted checkout, URL token capture/removal, local storage, daily verification, cached offline state, invalid-license relock, and paste-to-restore. Studio enables longer retention and team rubric-pack links.
- Privacy and terms pages, no analytics, no external fonts/scripts, a service-worker shell, security headers, JSON logs, health endpoint, and graceful shutdown.
- A product-specific paper-cut visual system and original generated hero with prompt provenance and optimized WebP assets.
- A non-root multi-stage Docker image serving frontend and backend together on port 8080. Persist `/app/data` in production.

## Run and verify

```bash
npm ci
npm test
npm run build
cargo run
curl http://localhost:8080/api/health
```

`npm run build` produces `dist/index.html`. `npm test` runs Vitest and the Rust API integration tests.

Final local verification on 2026-08-27:

- `npm test`: pass — 2 frontend tests, 3 backend integration tests.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo build --release --locked`: pass; release binary 8.2 MB.
- Full browser journey: pass — teacher code/link creation, student submission, queue refresh, review, and student read-only state; zero console errors (`.factory/evidence/e2e.json`).
- Factory `verify-url.sh`: pass — title, `lang`, one `h1`, main landmark, alt text, labeled buttons, and zero console errors at desktop and 390 px (`.factory/evidence/verify.json`).
- axe-core WCAG 2 A/AA/2.1 AA: 0 violations, 18 rule groups passed (`.factory/evidence/axe.json`).
- Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1.7 s, FCP 1.2 s, TBT 0 ms, CLS 0.022 (`.factory/evidence/lighthouse.json`).
- Initial assets: 69.8 KB JavaScript (25.9 KB gzip), 17.6 KB CSS (4.9 KB gzip), 29 KB mobile hero / 79 KB desktop hero.
- Load smoke: 500 concurrent health requests in 2.339 s, 213.8 requests/s, zero failures.

## Known gaps and release notes

- Docker is not installed in the worker, so the Dockerfile could not be executed here. The same locked release build and runtime layout were validated directly; deployment should still run an image-build smoke.
- The factory must register the billing product and confirm the production price/return URL. For staging, set `VITE_BILLING_BASE_URL=https://pilot-api.sociobot.in`.
- There are no teacher accounts by design. Clearing browser storage loses the workspace credential even though server records remain; the UI warns teachers. Export does not restore a lost credential.
- Shared rubric packs are immutable snapshots, expire after 30 days, and skip codes already present in the recipient workspace.
- SQLite suits the initial single-instance deployment. Multi-replica rollout would require shared PostgreSQL or sticky storage.
