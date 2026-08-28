# Independent verification 3 — FAIL

**Candidate:** `4ab9efc79742118bd8eb59c4627def63d2342e7d` (`main`)

**Live URL:** <https://rubric-revision-loop.sociobot.in>

**Verified:** 2026-08-28 from a detached clean checkout. Product source was
not modified.

## Decision

**FAIL.** The previous deployment-identity and client-only entitlement defects
are fixed: live health and the service worker report the exact candidate SHA,
all candidate-built frontend artifacts match live byte for byte, and direct
unlicensed premium writes now return `403`.

The release still cannot pass its advertised freemium workflow. The live “Buy
Studio — $24 once” link points to a production billing endpoint that returns
`404`, so a new user cannot buy the paid unlock. The anonymous write API also
has neither a rate limiter nor a storage quota despite the backend security
contract, leaving the shared SQLite volume open to trivial storage exhaustion.

## Fresh evidence

| Area | Result |
| --- | --- |
| Clean checkout/install | Detached worktree at the exact candidate; `npm ci` passed with 0 vulnerabilities. `npm audit --omit=dev` also found 0 vulnerabilities. |
| Automated tests | `npm test` passed: 2 Vitest tests, 8 Rust API/integration tests, service-worker cache-key regression, and Docker identity-wiring regression. |
| Type/format/lint | `npm run lint` passed: `tsc --noEmit`, `cargo fmt --check`, and strict `cargo clippy --all-targets -- -D warnings`. No separate JS lint task exists. |
| Production build | `BUILD_SHA=4ab9… npm run build` and `BUILD_SHA=4ab9… cargo build --release --locked` passed. `dist/` contains 70,047-byte JS (26.02 KB gzip), 17,593-byte CSS (4.86 KB gzip), and a 28,962-byte mobile hero. Docker is unavailable in this verifier container, so the Dockerfile itself could not be executed. |
| Runtime contract/identity | The release binary started with an empty environment except `PORT`/`PATH`, created its default SQLite store, and returned the exact candidate SHA. Data survived a process restart. Live `/api/health` returns `4ab9efc79742118bd8eb59c4627def63d2342e7d`; live `sw.js` uses the same cache key. |
| Candidate/live match | SHA-256 matched exactly for built/live `index.html` (`18c605…`), JS (`8a2152…`), CSS (`e3319d…`), `sw.js` (`f500be…`), and mobile hero (`05c4da…`). |
| Core browser journey | Live Chromium 1.58.2: create a rubric, create a student link, submit checklist + before/after + explanation, refresh queue, compare evidence, mark reviewed, and observe the student slip become read-only. Cleanup deleted the workspace with `204`. Healthy journey had 0 console errors, page errors, or failed requests. |
| Invalid/recovery/boundaries | Verified invalid workspace `401`; invalid rubric `422`; duplicate code `409`; unknown rubric `422`; 13 code slots `422`; retention 6 `422`; body over 64 KB `413`; incomplete/duplicate checklists `422`; reviewed resubmission `409`; reopen/resubmit succeeds; linked-code deletion `409`; unconfirmed workspace deletion `422`; confirmed deletion `204`; deleted link `404`. Minimum accepted field lengths and the 365-day boundary were exercised. |
| Premium enforcement | Live unlicensed 365-day loop and pack creation both return `403`; an invalid supplied license also returns `403`. The billing verify API returns `{valid:false, reason:"invalid"}` for the probe. The purchase endpoint itself returns `404` (defect below). A valid paid happy path could not be exercised because purchase is unavailable and no issued license exists. |
| Persistence/concurrency | Local data survived server restart. A live submitted/reviewed loop was returned by the teacher queue before cleanup. 100 parallel live health requests all returned `200` with the candidate SHA in 245 ms. |
| Accessibility/keyboard | Skip link is first Tab stop, visibly focused with a 3 px ring, and Enter focuses `#main`. Submission moves focus to the student status. Axe WCAG 2 A/AA/2.1 AA found 0 violations on live teacher, student, and 390 px screens and local privacy/terms screens. Reduced-motion media matched and reduced transition/animation durations to `0.00001s`. Mobile heading/target defects remain below. |
| Responsive/visual | Desktop and 390×844 layouts were visually inspected against the paper-cut design thesis. No horizontal overflow (`390 == scrollWidth`), clipped content, or generic framework treatment was found. |
| PWA/offline/update | On live, a fresh worker activation removed an injected obsolete cache and retained only `rrl-shell-4ab9…`. A controlled offline reload rendered the shell/main plus the explicit offline/reconnect state. The mobile heading defect also exists offline. |
| Privacy/outbound | The healthy teacher/student journey requested only the product origin. Static review found no analytics, trackers, CDN fonts/scripts, or model calls; only same-origin APIs and the documented Sociobot billing origin exist. Student responses omit `student_label`; cross-origin preflight receives `405` with no ACAO. `/privacy`, `/terms`, export, and confirmed deletion work. Expired writing retention remains a defect below. |
| Headers/cache | HTTP redirects to HTTPS. API responses are `no-store`; HTML and `sw.js` are `no-cache`; hashed assets are `public, max-age=31536000, immutable`. CSP, `nosniff`, DENY framing, strict-origin referrer policy, and request IDs are present. HSTS is absent. |
| Performance | Lighthouse 12.8.2 mobile: Performance 97, Accessibility 100, Best Practices 100, SEO 92; FCP 1.4 s, LCP 1.5 s, TBT 200 ms, CLS 0.022. Initial JS/CSS/hero all pass their budgets; Lighthouse reported no third parties. Lab Lighthouse does not produce a field INP value. |
| Library/CLI | Not applicable; this is a web-with-backend product. |

## Defects

### P1 — production Studio checkout is unavailable

The rendered button links to the contractually correct URL:

```text
https://api.sociobot.in/api/v1/products/rubric-revision-loop/checkout
```

A fresh `GET` (and `HEAD`) returns:

```http
HTTP/2 404
{"error":"enabled factory product","status":404}
```

The UI advertises a `$24 one-time` unlock, but no new user can purchase it.
This blocks the product's stated monetization and prevents paid-path end-to-end
verification. Enable/register the production factory billing product and then
verify checkout redirect, return `?license=`, token stripping/storage, server
authorization, restore, and revocation behavior.

### P1 — anonymous write API has no rate limiting or storage quota

The router applies only `DefaultBodyLimit`; there is no governor/rate-limit
layer or dependency and no per-workspace cap on rubric, loop, or pack rows.
Anyone can mint a syntactically valid workspace key and repeatedly write up to
the 64 KB request limit without an account. On the single shared SQLite volume,
this permits cheap disk exhaustion and conflicts with the required backend
rate-limiting baseline. Add server-side per-IP/workspace limits and bounded
resource quotas, with deterministic `429`/quota tests.

### P2 — expired links do not expire stored student writing

After moving a local 30-day loop 31 days into the past, student access correctly
returned `410`, but `/api/loops` still returned the submitted record and
`/api/export` still contained its private before/after text. Source inspection
shows no scheduled or request-time purge. Thus “retention” controls only link
access, while student writing remains indefinitely until a teacher explicitly
deletes the link/workspace. Purge expired student content or state this clearly
and provide criterion-level deletion consistent with data minimization.

### P2 — the 390 px teacher workspace has no accessible level-one heading

There is one DOM `<h1>`, but it is inside `.sidebar-title`, which becomes
`display:none` below 760 px. Chrome's 390 px accessibility tree contains only
the two level-two headings “Turn a rubric reason into a revision” and “Your
rubric library is empty”; it contains no level-one heading. Keep an accessible
page-level h1 in the mobile layout.

### P2 — mobile header/footer targets miss the 44×44 px baseline

At 390 px the home/brand link measures 195×42 px, Privacy 47×20 px, and Terms
39×20 px. Axe has no serious/critical finding for these sizes, but they fail the
attached non-negotiable 44 px touch-target contract. Increase their target boxes
without changing visual density.

### P3 — production responses omit HSTS

The HTTP origin redirects to HTTPS and other security headers are strong, but
live HTTPS responses do not include `Strict-Transport-Security`. Add HSTS at the
edge after confirming all subdomains are HTTPS-capable.

## Required re-verification

Enable the production Sociobot billing product, add API rate/resource limits,
and repair the mobile heading and touch targets. Clarify or implement actual
student-content retention. Then repeat the purchase/return/restore flow with a
real test license, direct API limit probes, the 390 px accessibility-tree and
target measurements, expiry/purge checks, and the complete live revision loop.
