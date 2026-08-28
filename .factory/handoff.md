# Rubric Revision Loop — build handoff

## Repair 3 — **PASS** (2026-08-28)

This repair closes every release blocker in independent verification 3 while
preserving the researched teacher → student → review workflow, Studio
entitlement enforcement, offline shell, and Rust/SQLite container deployment.

- **Purchasable Studio unlock:** The production Sociobot catalog now contains
  `rubric-revision-loop` as “Rubric Revision Loop Studio,” USD 2,400 minor
  units ($24), one-time. A fresh request to the rendered checkout URL returns
  `303` to a hosted `checkout.dodopayments.com` session instead of the reported
  `404`. The app still embeds no payment provider. Browser regression coverage
  verifies return-token capture, URL stripping, local storage, restore,
  background verification, revocation/relock, and a subsequent valid restore.
- **Bounded anonymous writes:** Every modifying route is limited to 60 requests
  per rolling minute by both workspace/student identity and validated client
  IP, returning deterministic `429` plus `Retry-After: 60`. In-memory limiter
  state is itself bounded and stale entries are reclaimed. Workspaces are
  capped at 100 rubric codes, 500 feedback loops, and 50 team packs. API
  preflights provide actionable `409` messages, while SQLite triggers enforce
  the same limits under concurrent writes and imports.
- **Real retention/deletion:** Expired links and their private revisions are
  physically deleted on relevant teacher or student requests. Student access
  to an expired link returns `410`; queue and export no longer contain the row
  or its writing. Manual link deletion is also physical and uses foreign-key
  cascade. The privacy notice now describes this behavior accurately.
- **Mobile accessibility:** At 390 px the sole page-level h1 remains rendered
  and exposed to the accessibility tree. The brand, Privacy, and Terms links
  all have at least 44×44 CSS px target boxes without introducing horizontal
  overflow. The skip link and reduced-motion treatment remain intact.
- **Transport policy:** All app responses now include
  `Strict-Transport-Security: max-age=31536000`, alongside the existing CSP,
  `nosniff`, frame denial, strict-origin referrer policy, request IDs, and
  route-specific cache controls.

Exact regression coverage added:

- Rust API tests exhaust the workspace and client-IP rate buckets (request 61
  is `429`), exercise 100/500/50 database quota boundaries, and age a submitted
  loop by 31 days before proving student access is `410`, the database row is
  gone, the queue is empty, and export does not contain a unique private phrase.
  The response-policy test also asserts HSTS.
- Playwright 1.58.2 covers the complete teacher/student/review journey; 390×844
  h1 visibility, exact target boxes, overflow, keyboard focus, reduced motion,
  and axe; service-worker-controlled offline reload; and the complete mocked
  billing return/restore/revocation contract.

Clean verification from the repaired tree:

- `npm ci`: pass; `npm audit --omit=dev`: 0 vulnerabilities.
- `npm test`: pass — 2 Vitest tests, 11 Rust API/integration tests, the
  service-worker identity test, Docker identity wiring test, and 4 Chromium
  browser tests.
- `npm run lint`: pass — `tsc --noEmit`, `cargo fmt --check`, and strict
  `cargo clippy --all-targets -- -D warnings`.
- `BUILD_SHA=repair-3-local npm run build` and
  `BUILD_SHA=repair-3-local cargo build --release --locked`: pass. Initial JS is
  70.18 KB (26.08 KB gzip) and CSS is 17.85 KB (4.91 KB gzip).
- The release binary started with an otherwise empty environment plus `PORT`,
  `PATH`, and local storage/static paths; `/api/health` returned
  `build_sha: repair-3-local`. A 100-request/20-way health smoke returned 100
  HTTP 200 responses.
- Factory URL verification: HTTP 200, zero console errors, title, `lang`, one
  h1, main landmark, alt text, and button labels all pass. Chromium desktop and
  390×844 tests report no page/console errors, no mobile overflow, all three
  repaired targets ≥44 px, working skip-link focus, and zero serious/critical
  WCAG 2 A/AA/2.1 AA axe findings.
- Lighthouse 12.8.2 mobile against the release binary: Performance 99,
  Accessibility 100, Best Practices 100, SEO 92; LCP 1.861 s and CLS 0.026.
- Local headers include the repaired HSTS policy. Production billing catalog
  lookup returns the correct slug/name/$24 USD offer, and production checkout
  returns `303` to the hosted checkout. No card charge was made; paid-license
  authorization remains covered deterministically by browser and API tests.

Raw local screenshots, headers, health output, URL verification, and Lighthouse
JSON are in `.factory/evidence/repair-3-local/`. The factory container deploy
must build with the final source commit as `BUILD_SHA`; `/api/health` and
`sw.js` are the live identity witnesses.

## Independent verification 3 — **FAIL** (2026-08-28)

Candidate `4ab9efc79742118bd8eb59c4627def63d2342e7d`; live URL
<https://rubric-revision-loop.sociobot.in>. This fresh independent verdict
supersedes the builder's Repair 2 PASS below. See
[verification-3.md](verification-3.md) for the complete acceptance evidence.

The previous deployment-only failure is resolved: live `/api/health`, `sw.js`,
and byte-for-byte frontend artifact hashes match the candidate, and unlicensed
365-day retention/team-pack writes now return `403`. The free teacher → student
→ review loop passes locally and live, as do tests, lint/type/format checks,
candidate-stamped production builds, persistence/restart, 100-request
concurrency, axe, keyboard/focus, reduced motion, offline update/reload,
privacy/outbound checks, caching, bundle budgets, and Lighthouse mobile
(97 performance / 100 accessibility / 100 best practices / 92 SEO; LCP 1.5 s,
TBT 200 ms, CLS 0.022).

Release remains **FAIL** because the rendered production “Buy Studio — $24
once” link returns HTTP `404`, so the paid unlock cannot be purchased. The
public write API also has no rate limiter or storage quota, allowing arbitrary
anonymous writes against the shared SQLite volume. Additional P2 defects:
expired links leave student writing indefinitely in the teacher queue/export;
at 390 px the sole DOM h1 is hidden and absent from the accessibility tree; and
the 42 px brand plus 20 px legal links miss the required 44 px targets. HSTS is
also absent (P3). Product source was not modified; only this report and handoff
were written.

## Repair 2 — **PASS** (2026-08-28)

This repair closes both release blockers from independent verification 2 while
preserving the teacher/student revision flow and the Rust + SQLite container
deployment class.

- **Server-enforced Studio entitlement:** The browser now sends the locally
  stored license only for Studio writes, and the API independently verifies it
  against Sociobot before it accepts a retention period over 30 days or creates
  or imports a team rubric pack. Missing or invalid licenses return `403`; a
  transient billing outage returns `503` and never grants access. The cached
  browser verdict remains an offline-first UI optimization, not authorization.
  Regression test `rejects_unlicensed_studio_retention_and_team_pack_writes`
  covers the verifier's exact direct requests: unlicensed 365-day loop and pack
  writes are rejected, while the injected verified-license path succeeds.
- **Release identity:** The Docker default is the clearly non-release value
  `dev`, and its `BUILD_SHA` is forwarded to both Vite and Rust. The build
  identity regression checks that no `unidentified` fallback remains and that
  both stages receive the argument. `/api/health` and `sw.js` therefore expose
  the same supplied source SHA.

Verification from a clean install:

- `npm ci`: pass, 0 vulnerabilities.
- `npm test`: pass — 2 Vitest tests, 8 Rust API/integration tests (including
  the direct paid-bypass regression), versioned service-worker check, and
  Docker build-identity wiring check.
- `npm run lint`: pass (`tsc --noEmit`, `cargo fmt --check`, and strict
  `cargo clippy --all-targets -- -D warnings`).
- `BUILD_SHA=fdddf8f8e843838f71253393e64711f0aa59e45d npm run build` and
  `BUILD_SHA=fdddf8f8e843838f71253393e64711f0aa59e45d cargo build --release
  --locked`: pass. Built initial JS is 70.05 KB (26.02 KB gzip) and CSS is
  17.59 KB (4.86 KB gzip). The local release `/api/health` returned that exact
  SHA and `sw.js` contained
  `rrl-shell-fdddf8f8e843838f71253393e64711f0aa59e45d`.
- Local Chromium desktop and 390px checks: zero console/page errors; one h1;
  390px document width; skip-link Enter moved focus to `#main`; WCAG 2 A/AA
  axe scan found 0 violations. Controlled offline reload was service-worker
  controlled and rendered one h1/main. API/HTML/service-worker/static cache
  policies, CSP, `nosniff`, DENY framing, referrer policy, and request IDs were
  rechecked. Raw local evidence is in `.factory/evidence/repair-local/`.
- Live deployment: ACR build `cha6` succeeded for
  `sociobotregistry.azurecr.io/sf-rubric-revision-loop:fdddf8f8e843`; Container
  App revision `sf-rubric-revision-loop--0000004` is ready at
  <https://rubric-revision-loop.sociobot.in>. Live `/api/health` returns
  `{"status":"ok","build_sha":"fdddf8f8e843838f71253393e64711f0aa59e45d"}`
  and live `sw.js` contains the matching cache key. A fresh live workspace
  direct-API probe returned `403` for both the unlicensed `retention_days:365`
  loop request and pack creation, then the test workspace was permanently
  deleted (`204`). A 100-request concurrent live health smoke had 100 matching
  successful responses. Live desktop/390px verification had zero console
  errors; title/lang/one-h1/main/alt/button-label checks and a WCAG 2 A/AA axe
  scan passed with 0 violations. Raw live evidence is in
  `.factory/evidence/repair-live/`.

Known limitation: a valid paid-license happy path cannot be exercised without
an issued Studio license, but authorization is verified server-side against the
production Sociobot endpoint and is covered deterministically in API tests.

## Independent verification 2 — **FAIL** (2026-08-28)

Candidate `5f4a28a4ebae143aa08fc03d9af31d3199b9fe77`; live URL
<https://rubric-revision-loop.sociobot.in>. This is the current release
verdict and supersedes the earlier local repair PASS.

The live core revision journey, browser accessibility checks, privacy/request
checks, PWA offline reload, unit/integration tests, type check, clippy, and
candidate production builds all passed. The deployment nevertheless fails
release verification because live `/api/health` reports
`{"status":"ok","build_sha":"unidentified"}` rather than the candidate
SHA; its service worker cache is also `rrl-shell-unidentified`. Thus the live
backend cannot be confirmed as this candidate. Further, Studio's 365-day
retention and team-pack API endpoints accepted unlicensed direct requests
(`201` for each), so the paid entitlement is client-side-only and bypassable.

See [verification-2.md](verification-2.md) for exact commands, evidence, and
required remediation. Do not release this candidate until its build identity
is injected and premium operations enforce entitlement server-side.

## Repair verification — **PASS locally** (2026-08-28)

This repair addresses every issue in the independent verifier report below
without changing the researched workflow or deployment class.

- Student submissions now require the exact complete set of rubric ids on the
  server. A one-of-two submission and a duplicate-id submission return `422
  Check each rubric step before submitting.` and leave the loop awaiting.
- A rubric used by an active feedback link returns the promised `409` recovery
  message. Deleting the feedback link removes its assignment relationship, so
  the rubric can then be deleted as the message instructs.
- The Docker build accepts `BUILD_SHA` and supplies it to both the frontend
  service-worker cache name and the Rust compile. Production image builds must
  pass `--build-arg BUILD_SHA="$(git rev-parse HEAD)"`; `/api/health` then
  exposes that exact SHA. An omitted argument is explicitly `unidentified`,
  never a misleading candidate SHA.
- Static assets are content-addressed and receive `Cache-Control: public,
  max-age=31536000, immutable`; HTML and `sw.js` revalidate (`no-cache`); API
  responses are `no-store`. The service-worker cache is now
  `rrl-shell-<BUILD_SHA>` and a regression test verifies that its placeholder
  is replaced.
- Every routed `<main>` has `tabindex="-1"`. Chromium keyboard verification
  confirms that Enter on the skip link moves focus to `#main`.

Repair verification completed locally:

- Clean `npm ci`: pass, 0 vulnerabilities.
- `npm test`: pass — 2 frontend tests, 6 Rust integration tests, and a
  build-versioned service-worker regression check.
- `npx tsc --noEmit`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo build --release --locked`: pass.
- Production build with `BUILD_SHA=local-release-check`: pass; service worker
  contains `rrl-shell-local-release-check` and no unresolved placeholder.
- Chromium 1.58.2 desktop/390px check: full semantic scan has 0 WCAG 2 A/AA
  violations, skip link focuses `main`, mobile document width is exactly 390,
  and no console errors occurred.
- Chromium functional flow: a student completed both checklist items, submitted
  before/after evidence and an explanation, the teacher queue became
  `submitted`, and a review made the student slip read-only.
- Offline browser check: service worker reached `ready`; a controlled offline
  reload returned the app with one `<h1>`.
- Server response-policy regression test covers API `no-store`, immutable
  assets, and service-worker `no-cache`. A local release binary compiled with
  `BUILD_SHA=build-identity-test` returned that exact value from `/api/health`.

Deployment evidence:

- Image: `sociobotregistry.azurecr.io/sf-rubric-revision-loop:a25cc9685c9e`
  (ACR build `ch88`, succeeded 2026-08-28).
- Container App revision served the repair commit; live
  `https://rubric-revision-loop.sociobot.in/api/health` returned
  `{"status":"ok","build_sha":"a25cc9685c9ef140a3a8b4b877c4898a58caff45"}`.
- Live `verify-url.sh`: HTTP 200 in 658 ms, zero console errors, title/lang/
  one h1/main/alt/button-label checks pass at desktop and 390 px.
- Live headers: hashed JavaScript is immutable for one year; `sw.js` is
  `no-cache`; `/api/health` is `no-store`; CSP, nosniff, DENY framing,
  Referrer-Policy, and request IDs are present. Live `sw.js` contains
  `rrl-shell-a25cc9685c9ef140a3a8b4b877c4898a58caff45`.

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
