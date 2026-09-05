# Independent review: reuse rubric feedback and review revisions — FAIL

**Verdict: FAIL**

- Findings: **10** (4 P1, 4 P2, 2 P3)
- Untested public claims: **14**
- Implementation reviewed: `f02906400387d14e6866a55ef890b74a198ea846`
- Documentation HEAD reviewed: `c38fccec5fe4ebb96ee9e895ddc9ff88e3305204`
- Live URL: <https://rubric-revision-loop.sociobot.in>
- Review date: 2026-09-05 UTC

The complete teacher, student, and review workflow works. The product still
fails the required demo, claims, accessibility, routing, rate-limit, and
durable-storage contracts. Product code was not changed during this review.

## First screen before scrolling

- **Job:** A teacher reuses criterion-specific feedback, sends a student link,
  and reviews the student's before-and-after revision evidence.
- **Audience:** Writing teachers with a large repeated-feedback load. This is
  inferred from “Teacher desk”; the page does not state the audience in a
  complete sentence.
- **First action:** Desktop eventually offers “Create your first code.” At
  390×844 that action is below the first screen. The visible “Create feedback”
  tab leads to the same empty state. There is no “Try it with sample data”
  action.

The h1, “Make the next revision visible,” does not name the concrete job. The
document title is only “Rubric Revision Loop.” The first screen has no sample
path, no sentence that names the audience, no explanation beside a primary
action, and no three plain facts.

## Candidate and live identity

`f029064…` is the last source-changing commit; it added the root health route
and its response-policy test. `c38fcce…` adds only independent verification
reports and evidence. Live `/api/health`, `/health`, and the service-worker
cache all report `f029064…`. Candidate-built `index.html`, JS, and CSS match
live byte for byte:

| Artifact | SHA-256 result |
| --- | --- |
| `index.html` | match, `bb2fec39…` |
| JS | match, `4f7c7dc5…` |
| CSS | match, `3fa573e0…` |
| Mobile hero | match, `05c4da3f…` |

## Clean checkout commands

The commands ran in a detached clean worktree at documentation HEAD.

| Command | Result |
| --- | --- |
| `npm ci` | PASS; 81 packages installed, 0 vulnerabilities |
| `npm audit --omit=dev` | PASS; 0 vulnerabilities |
| `npm test` | PASS; 2 Vitest, 11 Rust, and 4 Playwright tests |
| `npm run lint` | PASS; TypeScript, rustfmt, and strict clippy |
| `BUILD_SHA=f029064… npm run build` | PASS; `dist/` produced |
| `BUILD_SHA=f029064… cargo build --release --locked` | PASS |
| Declared claim commands | None exist; `.factory/claims.json` is missing |
| Docker build | Not run; Docker, Podman, and Buildah are unavailable |

The build contains 70,182 B JS (26.08 KiB gzip), 17,845 B CSS (4.91 KiB
gzip), and a 28,962 B mobile hero. These pass the repository size budgets.

## Live review results

| Area | Result |
| --- | --- |
| Fresh desktop and phone | Opened in separate empty Chromium contexts at 1440×900 and 390×844. No console or page errors; no horizontal overflow. Screenshots are in `.factory/evidence/review-1/`. |
| Sample path | FAIL. No sample action exists. `/demo` opens the empty normal workspace, creates the normal `rrl_workspace_key`, and reuses it at `/`. There is no demo label, reset, start-for-real action, isolated namespace, or `.factory/demo.md`. No demo write was attempted because it would use real storage. |
| Real workflow | PASS. Created two realistic rubric codes, a private student link, two checked criteria, focused before/after excerpts, and an explanation. The teacher queue showed the complete comparison, review made the student page read-only, and export contained two rubrics plus one loop. The workspace was deleted with `204`; its student link then returned `404`. |
| Invalid and boundary input | PASS for bad workspace `401`, malformed rubric `422`, duplicate code `409`, 6-day retention `422`, unlicensed 365-day retention `403`, 13 criteria `422`, incomplete checklist `422`, reviewed resubmission `409`, linked-code deletion `409`, and body over 64 KiB `413`. |
| Recovery | PASS. Reopen then resubmit returned `200`. Invalid browser input focused the field and said “Please match the requested format.” |
| Tenant isolation | PASS. A second workspace listed no first-workspace rubrics and could not delete one (`404`). Student JSON omitted the teacher's student label. |
| Privacy controls | PASS. JSON export and permanent workspace deletion worked. The main journey contacted only the product origin. No analytics, ads, external fonts/scripts, or model calls were observed. Privacy and terms routes return `200`. |
| Health and restart | PASS locally with only `PATH` and `PORT`: the release server started and a rubric survived a process stop/restart. Both live health routes return the candidate SHA; 100 parallel live health requests returned `200`. This does not repair the `/data` mount defect below. |
| Rate limits | PARTIAL. A live write burst reached `429` with `Retry-After: 60`. A 75-request authenticated read burst returned 75×`200` with no limit. Source keys forwarded clients from the last address, contrary to the first-hop contract. |
| Keyboard and focus | PASS except the accessible-name finding. Skip link is first, has a 3 px visible ring, Enter focuses `main`, all nine empty-workspace controls are reachable, and Space changes to the rubric library. There was no trap. |
| Resize and touch | PASS. At 200% text size the 390 px page retained content and had no horizontal overflow. Mobile navigation and legal targets are at least 44 px. |
| Reduced motion | PASS. Reduced-motion transitions and animations compute to `0.00001s`; nothing loops. |
| Accessibility scan | FAIL. Axe 4.13 with WCAG 2.5.3 enabled reports one serious `label-content-name-mismatch` on `.brand` at desktop, phone, privacy, terms, invalid student, and unknown routes. Other axe rules, landmarks, labels, image alt text, contrast, and heading counts pass. |
| Offline and update | PASS for the stated shell behavior. A fresh worker used `rrl-shell-f029064…`; controlled offline reload kept the h1 and showed the reconnect message. |
| Links and billing | Internal home/privacy/terms links work. The production Studio link returns `303` to the hosted checkout. No payment was made, so issued-license purchase, refund, and revocation remain unverified live. |
| Routes and 404 | FAIL. Privacy, terms, unknown routes, and invalid student routes all keep the same generic title. An unknown page returns `200` and the normal workspace, not a designed 404. API unknown routes correctly return deliberate `404`; that is expected and is not a defect. |
| Performance | Lighthouse 12.8.2 mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 92; FCP 1.4 s, LCP 1.5 s, TBT 10 ms, CLS 0.026. The separate explicit axe rule still fails. |

## Public claims

There is no `.factory/claims.json` and no `@claim:` test in the repository.
The following 14 distinct claims appear in the UI, legal pages, or README.
Independent observations below do not replace the required tagged claim test.
All 14 therefore count as untested public claims under the claims contract.

| # | Public claim | Independent observation |
| --- | --- | --- |
| 1 | Rubric codes lead to a student revision and teacher review queue | Observed live |
| 2 | Feedback and grades are not generated | Static/runtime review found no model call; operational promise remains unregistered |
| 3 | No account or email is required; a random browser key controls the workspace | Observed live and in source |
| 4 | A student link reveals only its selected criteria and submission | Isolation probe passed |
| 5 | Free links expire after 30 days and their writing is deleted | Rust expiry test passed; no tagged claim command |
| 6 | Export and permanent workspace deletion are available to everyone | Observed live |
| 7 | Studio costs $24 once and provides packs plus 90/365-day retention | Checkout redirect observed; issued paid flow not exercised |
| 8 | The server verifies licenses before paid writes and revoked licenses relock | Unlicensed live write rejected; valid/revoked cases are mocked only |
| 9 | Anonymous writes are limited to 60 changes per minute per client and workspace | A write limit exists; forwarded-client and full-endpoint behavior is incomplete |
| 10 | Workspaces are capped at 100 rubrics, 500 links, and 50 packs | Repository boundary test passed |
| 11 | No analytics, ads, third-party fonts/scripts, or runtime CDNs load | Live request log and static review passed |
| 12 | Student writing is not sold, advertised against, or used for model training | Policy statement cannot be proved from this repository alone |
| 13 | Only the license token is stored locally; card details never reach this service | Source supports it; no real purchase was completed |
| 14 | Saved pages remain readable offline and submissions require a connection | Observed in a fresh service-worker context |

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Incomplete student checklist accepted | FIXED: live incomplete submission is `422`; complete submission is `200`. |
| Backend and service-worker identity missing | FIXED: both identify `f029064…`. |
| Linked rubric deletion returned `500` | FIXED: live response is recoverable `409`. |
| Static cache and worker generation were unsafe | FIXED: hashed assets are immutable and worker cache includes the candidate SHA. |
| Skip-link target did not receive focus | FIXED: Enter focuses `main`. |
| Premium writes bypassed browser controls | FIXED for tested unlicensed writes: 365-day creation is `403`. |
| Production checkout returned `404` | FIXED: it returns `303` to hosted checkout. |
| No write limits or quotas | PARTLY FIXED: writes return `429` and quota tests pass, but reads are unlimited and forwarded-IP selection violates the current contract. |
| Expired links retained writing | FIXED by the current Rust expiry/purge test and source path. |
| Mobile h1 hidden and targets under 44 px | FIXED: h1 is visible; measured targets meet 44 px. |
| HSTS absent | FIXED: `Strict-Transport-Security: max-age=31536000`. |
| Brand accessible name mismatch | OPEN: serious axe failure reproduced on every tested route. |
| `/robots.txt` returned the SPA document | OPEN; `/sitemap.xml` has the same problem. |
| JS and CSS uncompressed | OPEN; Lighthouse still estimates 56 KiB savings. |

## Findings

### P1 — no one-click sample or isolated demo

There is no visible sample action. `/demo` is only the ordinary empty
workspace and uses the same local-storage key as `/`. It has no realistic
populated data, persistent “Demo — sample data, nothing is saved” label, reset,
start-for-real action, or separate storage tenant. `.factory/demo.md` is also
missing. This blocks the required safe path for testing value before setup.

### P1 — public claims have no claim registry or tagged tests

`.factory/claims.json` is missing and `rg '@claim:'` returns no matches. The 14
claims listed above therefore have no declared command that can be run from a
clean checkout. Some happen to be covered by general tests or this review, but
none meets the claim contract and several paid/operational promises remain
unverified end to end.

### P1 — product state is not written to the fleet `/data` mount

The server default is `sqlite://data/revision-loop.db`, while the image sets
`DATABASE_URL=sqlite://data/revision-loop.db` under `/app` and creates
`/app/data`. The README also tells operators to mount `/app/data`. The required
fleet mount is `/data`, so a production redeploy can replace the database even
though an ordinary process restart passed. The application must prefer
`/data/revision-loop.db` when `/data` exists.

### P1 — rate limiting does not cover every endpoint or use the required client address

The middleware limits only `POST`, `PUT`, `PATCH`, and `DELETE`. Seventy-five
live `GET /api/rubrics` requests all returned `200` without `Retry-After`.
Writes do reach `429` with `Retry-After: 60`, but source selects
`X-Forwarded-For.split(',').next_back()` instead of the required first hop.
Apply bounded per-client limits to every server endpoint except health and key
them from the validated first forwarded address.

### P2 — the WCAG 2.5.3 accessible-name mismatch remains

The brand shows `R↻ Rubric Revision Loop` or `R↻ Revision slip`, but
`aria-label="Rubric Revision Loop home"` replaces that visible label. Axe
reports `label-content-name-mismatch` with serious impact at desktop, phone,
legal, invalid-student, and unknown routes. The repository axe configuration
still omits this rule, so `npm test` passes while the required scan fails.

### P2 — the first screen and landing structure do not explain the job or provide a working first action

The generic h1 does not say “Reuse rubric feedback and review revisions” or an
equivalent concrete job. The audience is not stated in a sentence. On phone,
the useful “Create your first code” action is below the first 844 px, while the
visible active tab leads to an empty state. There is no landing order with a
sample action, live populated preview, three-step explanation, limits/privacy,
and paid tier.

### P2 — route titles, metadata, and 404 behavior do not meet the site contract

Every route uses “Rubric Revision Loop,” including privacy, terms, student, and
unknown routes. The root title does not name the job. Canonical, Open Graph,
Twitter card, favicon, Apple touch icon, and a 1200×630 product image are
absent. Unknown paths return `200` and create/open a workspace instead of
showing a designed 404 with a route-specific title and way home. The footer
also omits “Built by Param Factory” and the build id.

### P2 — the container build and startup reporting violate the backend contract

The Dockerfile pins `rust:1.88-slim-bookworm` instead of the required
`rust:1-slim` family. The startup JSON line reports the address and SHA but not
which runtime configuration was supplied or generated. No local container
engine was available, so the image itself could not be rebuilt; the locked
native Rust release build passed.

### P3 — robots and sitemap URLs return HTML

`/robots.txt` and `/sitemap.xml` both return `200 text/html` with the app shell.
Lighthouse reports 15 robots parse errors. Serve valid files with the correct
content types or an intentional 404 for robots; the required sitemap must list
the real routes.

### P3 — live text assets are not compressed

Requests offering Brotli and gzip receive neither `Content-Encoding` on the
70,182 B JS nor on the 17,845 B CSS. Lighthouse estimates 56 KiB avoidable
transfer. Current loading scores and hard bundle sizes still pass.

## Evidence and cleanup

Fresh browser, API, accessibility, screenshots, URL verification, and
Lighthouse evidence is under `.factory/evidence/review-1/`. The local runtime
used only synthetic data. Both live synthetic workspaces were deleted; the
student links returned `404` after cleanup. No payment was made. No product
code, deployment, infrastructure, DNS, billing configuration, or secrets were
changed.
