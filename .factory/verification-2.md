# Independent verification 2 — FAIL

**Candidate:** `5f4a28a4ebae143aa08fc03d9af31d3199b9fe77` (`main`)

**Live URL:** <https://rubric-revision-loop.sociobot.in>

**Verified:** 2026-08-28 from a clean checkout. Product source was not modified.

## Decision

**FAIL.** The ordinary revision loop is sound, but this deployment cannot be
identified as the candidate: live `/api/health` returns
`{"status":"ok","build_sha":"unidentified"}` rather than the candidate
SHA. The live service worker is likewise `rrl-shell-unidentified`, while the
candidate production build emits `rrl-shell-5f4a28a4ebae143aa08fc03d9af31d3199b9fe77`.
The requested live/candidate confirmation is therefore impossible.

In addition, paid Studio controls are only hidden in the browser. A caller
without a license can directly create both a 365-day link and a team rubric
pack through the public API. This defeats the stated freemium entitlement.

## Fresh evidence

| Area | Result |
| --- | --- |
| Clean install | `npm ci` passed; npm reported 0 vulnerabilities. |
| Tests | `npm test` passed: 2 Vitest tests, 7 Rust tests, and the service-worker cache-key regression test. |
| Type/lint | `npx tsc --noEmit` and `cargo clippy --all-targets -- -D warnings` passed. There is no repository JS lint script. |
| Production build | `BUILD_SHA=5f4a… npm run build` passed. `BUILD_SHA=5f4a… cargo build --release --locked` passed. The local release server returned the exact candidate SHA from `/api/health`. Docker is unavailable in this environment, so the image itself could not be constructed. |
| Core live journey | Chromium desktop: created two codes, created a link, completed both student checklist steps plus before/after excerpts and explanation, refreshed the queue, marked it reviewed, and observed the student slip become read-only. No console or page errors. |
| Invalid/recovery | Invalid rubric request `422`; duplicate code `409`; incomplete two-code student checklist `422`; complete checklist `200`; linked-code deletion `409` with recovery guidance; unconfirmed workspace delete `422`, confirmed delete `204`, then student link `404`. A separate workspace export contained 0 rubrics and 0 loops. |
| Boundary/paywall API probe | An unlicensed `POST /api/loops` with `retention_days:365` returned `201`; an unlicensed `POST /api/packs` returned `201`. |
| Concurrency/persistence | 100 concurrent live `/api/health` requests all completed successfully. Data created through the API remained available until the explicit workspace deletion. |
| Accessibility/browser | Live mobile Chromium at 390px had `scrollWidth: 390`; Tab focused the visible skip link and Enter moved focus to `#main`. Reduced-motion transition duration was `0.00001s`. Axe WCAG 2 A/AA found 0 violations, hence 0 serious/critical findings. |
| PWA | Service worker reached ready state and a controlled offline reload rendered the app with its one h1. The cached live worker uses the non-candidate `unidentified` generation. |
| Privacy/outbound | The exercised core journey made only same-origin requests; no third-party fonts, scripts, analytics, or trackers were observed. Static inspection found only same-origin API calls plus the documented Sociobot billing origin. Student API responses omit the teacher-only `student_label`; `/privacy` and `/terms` are present. |
| Headers/cache | Live HTML and `sw.js` are `no-cache`; API is `no-store`; hashed JS/CSS are `public, max-age=31536000, immutable`. CSP, `nosniff`, `X-Frame-Options: DENY`, strict-origin referrer policy, and request IDs are present. |
| Budget | Built initial JS: 69,853 bytes / 25,838 gzip; CSS: 17,593 / 4,857 gzip; mobile hero: 28,962 bytes. All are within the stated static budgets. |
| Live comparison | Candidate-built `index.html`, JS, and CSS SHA-256 values exactly match live. Candidate SW SHA-256 `cc4795…` does **not** match live `fafebc…`: only the build cache identifier differs (`5f4a…` versus `unidentified`). Backend identity cannot be compared because live health reports `unidentified`. |

## Defects

### P1 — live deployment is not verifiably candidate `5f4a28a`

`GET /api/health` at the release URL returned `build_sha: "unidentified"`.
The exact local release binary, compiled with `BUILD_SHA=5f4a…`, returns the
candidate SHA. The same omission produces live `const CACHE =
'rrl-shell-unidentified'`. This is a deployment/release-identity failure and
blocks confirmation that the live backend is the candidate.

### P1 — paid Studio features are trivially bypassed at the API

With only a syntactically valid random workspace key and no license, the live
API accepted:

```http
POST /api/loops
{"assignment_title":"Paid bypass","rubric_ids":[1],"retention_days":365}
→ 201 Created

POST /api/packs
{"rubric_ids":[1,2]}
→ 201 Created
```

The UI hides these controls until client-side license state says unlocked, but
the server has no entitlement check. Longer retention and shared packs are the
advertised paid unlock, so this breaks the freemium contract.

### P2 — service-worker cache generation is not safely deploy-versioned in live

The cache key is structurally versioned, but the deployed value is the constant
`unidentified`. Any subsequent deployment made without the missing build SHA
would reuse that cache generation, preventing deterministic update validation.
This follows from the P1 deployment configuration issue.

## Required re-verification

Deploy the candidate with `BUILD_SHA=5f4a28a4ebae143aa08fc03d9af31d3199b9fe77`
passed to the image build, then confirm the live health endpoint and service
worker cache key. Enforce Studio entitlement server-side (or make the premium
operations unavailable without a verifiable server-side entitlement) and
repeat the direct API bypass checks plus the live revision journey.
