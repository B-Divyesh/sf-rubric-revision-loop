# Independent verification 4 — FAIL

**Candidate:** `f02906400387d14e6866a55ef890b74a198ea846` (`main`)

**Live URL:** <https://rubric-revision-loop.sociobot.in>

**Verified:** 2026-08-28 from a clean candidate checkout. Product source was
not modified.

## Decision

**FAIL.** The deployed backend and every compared frontend artifact match the
candidate, the complete teacher → student → review loop works, and all five
defects from independent verification 3 are repaired. The production checkout
now redirects to a hosted Dodo session, anonymous writes are bounded, expiry
physically removes writing, the mobile h1 and 44 px targets are present, and
HSTS is served.

Release acceptance still fails the supplied non-negotiable accessibility gate.
Lighthouse and an explicit axe WCAG 2.5.3 scan report one **serious**
`label-content-name-mismatch` violation on the persistent brand/home link at
both desktop and 390 px. The acceptance contract requires all serious/critical
axe findings to be fixed.

## Fresh evidence

| Area | Result |
| --- | --- |
| Clean checkout/install | Working tree began clean at the exact candidate, which is also `origin/main`; `npm ci` passed and `npm audit --omit=dev` found 0 vulnerabilities. |
| Repository test gate | `npm test` passed: 2 Vitest tests, 11 Rust API/integration tests, service-worker/build-identity checks, and 4 Playwright journeys. |
| Type/lint/format | `npm run lint` passed: `tsc --noEmit`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings`. No separate JavaScript linter exists. |
| Exact builds | `BUILD_SHA=f029064… npm run build` and `BUILD_SHA=f029064… cargo build --release --locked` passed. The Dockerfile uses those same locked build commands and propagates `BUILD_SHA` to both stages, but no Docker/Podman/Buildah daemon or CLI is installed in this verifier container, so the image itself could not be rebuilt. |
| Runtime contract and identity | The release binary started with an empty environment except `PATH` and `PORT=18080`, created/used its default SQLite store, and returned the exact SHA. Data survived a stop/restart. Live `/api/health` returns the same SHA and live `sw.js` uses `rrl-shell-f029064…`. |
| Candidate/live match | SHA-256 matched byte for byte for `index.html` (`bb2fec…`), JS (`4f7c7d…`), CSS (`3fa573…`), `sw.js` (`7fcf03…`), and the 720 px hero (`05c4da…`). |
| Core end-to-end flow | Fresh live Chromium: create a rubric, create a student link, submit the checked criterion plus before/after evidence and explanation, refresh the teacher queue, compare evidence, mark reviewed, and confirm the student form becomes read-only. Normal browsing produced 0 console errors, page errors, or failed requests. The deterministic QA workspace was deleted (`204`). |
| Invalid/boundary/recovery paths | Fresh release API probes verified malformed rubric `422`, duplicate code `409`, retention 6 days `422`, unlicensed 365 days `403`, 13 criteria `422`, incomplete checklist `422`, too-short explanation `422`, reviewed resubmission `409`, reopen then resubmit `200`, linked-rubric deletion `409`, bad workspace key `401`, unconfirmed workspace deletion `422`, confirmed deletion `204`, and deleted student link `404`. Keyboard-only activation created a valid rubric; an invalid code moved focus to the field and exposed the browser's actionable format error. |
| Persistence/concurrency/limits | A submitted loop and corrected excerpt remained after a release-process restart. 100 local parallel health requests and 100 live requests at 20-way concurrency all returned `200`; every live response reported the candidate SHA (live mean 0.795 s, max 1.667 s). With a fresh IP/workspace bucket, writes 1–60 reached validation and write 61 returned `429` with `Retry-After: 60`. Rust tests independently cover 100-rubric, 500-loop, and 50-pack database quota boundaries. |
| Retention/privacy | The expiry integration test ages a submitted loop beyond retention and proves student access is `410`, the row is physically gone, and queue/export omit the private phrase. A healthy live journey contacted only the product origin. Static review found no analytics, ads, CDN fonts/scripts, or model calls; only documented Sociobot billing URLs exist. Student JSON omits the teacher-only student label. `/privacy`, `/terms`, export, and confirmed workspace deletion work. Cross-origin preflight returned `405` without ACAO. |
| Studio billing | The rendered buy link is exactly `https://api.sociobot.in/api/v1/products/rubric-revision-loop/checkout`; a fresh request returned `303` to `checkout.dodopayments.com`, fixing the prior `404`. Browser tests cover return-token capture, URL stripping, storage, restore, background verification, revocation, and re-restore with deterministic mocked verdicts. Direct unlicensed premium writes return `403`. No payment was made, so a real issued-license happy path was not exercised. |
| Keyboard/accessibility | The skip link is the first Tab stop, has a visible 3 px focus outline, and Enter focuses `#main`; successful student submission focuses its status. Pages have `lang=en`, a title, one visible h1, one main, labeled controls, and descriptive image alt. Standard axe 4.13 scans found 0 serious/critical findings on teacher, student, mobile, privacy, and terms views. The explicit WCAG 2.5.3 scan finds the serious defect below. |
| Mobile/reduced motion | At 390×844, the h1 is visible in the accessibility tree; document and scroll widths are both 390 px. Brand, Privacy, and Terms targets measure 44 px high (Terms is 44×44). Body text is 16 px. Reduced-motion transitions/animations resolve to `0.00001s`. Visual inspection found no overlap, clipping, or broken hierarchy. |
| PWA | A newly installed live service worker removed an injected obsolete cache and retained only `rrl-shell-f029064…`. A controlled offline reload rendered the h1/main and explicit “Offline” recovery status. The two `ERR_INTERNET_DISCONNECTED` console messages occurred only during the deliberate offline reload. |
| Headers/cache | HTTP redirects to HTTPS. API is `no-store`; HTML and `sw.js` are `no-cache`; hashed assets are `public, max-age=31536000, immutable`. CSP, HSTS (`max-age=31536000`), `nosniff`, frame denial, strict-origin referrer policy, and request IDs are present. |
| Performance/budgets | Candidate output is 70,182 B JS (25,967 B gzip), 17,845 B CSS (4,922 B gzip), and 28,962 B mobile hero. Lighthouse 12.8.2 mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 92; FCP 1.38 s, LCP 1.54 s, TBT 55 ms, CLS 0.0256, total transfer 119,287 B. Lighthouse does not produce a lab INP value. |
| Library/CLI | Not applicable; this is a web-with-backend PWA. |

## Defects

### P2 — brand link fails WCAG 2.5.3 label-in-name

The persistent link renders visible text `R↻ Rubric Revision Loop` but overrides
its accessible name with `Rubric Revision Loop home`:

```html
<a class="brand" href="/" aria-label="Rubric Revision Loop home">
  <span aria-hidden="true">R↻</span> Rubric Revision Loop
</a>
```

Lighthouse reports `label-content-name-mismatch` with impact `serious`, tagged
`wcag21a`/`wcag253`; explicitly enabling that axe 4.13 rule reproduces the same
single serious violation at 1440×900 and 390×844. This can make visible-label
voice activation unreliable. Standard axe runs omit this experimental rule,
which is why the repository test and the default scan incorrectly appear clean.
Align the accessible name with all visible text (or render the decorative mark
without text semantics), then keep this rule enabled in regression coverage.

### P3 — `/robots.txt` is the SPA document, not a robots policy

`GET /robots.txt` returns `200 text/html` containing `index.html`. Lighthouse
reports 15 parse errors and SEO 92. Serve a valid text/plain robots file (or an
intentional 404) rather than the application shell.

### P3 — text assets are not compressed in transit

Even with `Accept-Encoding`, live JS and CSS responses have no
`Content-Encoding`; Lighthouse estimates 56 KiB avoidable transfer. Current
assets remain under the hard bundle budget and measured performance is 100, so
this is not release-blocking by itself.

## Evidence and cleanup

Screenshots, the factory URL report, and Lighthouse JSON are under
`.factory/evidence/verification-4/`. All deterministic local/live test
workspaces were deleted. One first-pass browser harness aborted after creating
an unlabelled synthetic QA loop before its random browser key could be captured;
it contains no personal data and will be purged by the product's 30-day
retention path.

## Required re-verification

Repair the brand link's label-in-name mismatch and add the explicit WCAG 2.5.3
axe rule to browser coverage. Then rerun that scan at desktop/mobile and the
keyboard focus smoke. The robots and compression findings should be addressed
as release hardening.
