# Independent verification — FAIL

**Candidate:** `f9606e82711c234c724a65ca7ac00ed87d14cacb` (`main`)

**Live URL:** <https://rubric-revision-loop.sociobot.in>

**Verified:** 2026-08-27, from a clean worktree. Product code was not changed.

## Decision

**FAIL.** The ordinary teacher-to-student revision loop works, but the server
accepts a student revision with only a subset of its assigned rubric checklist.
This breaks the core promise that every rubric reason is addressed before the
teacher reviews the evidence. The live backend also reports `build_sha: "dev"`,
so its identity cannot be confirmed as the candidate commit.

## Reproduction of blocking defect

Create a feedback loop with two rubric ids, then submit this directly to its
student endpoint with only one id in `checklist`:

```json
{
  "before_excerpt": "Before.",
  "after_excerpt": "After.",
  "explanation": "I changed the sentence so it now includes a specific detail.",
  "checklist": [21]
}
```

Fresh release-build evidence:

```json
{"http_status":"200","stored_checklist":[21],"assigned":[21,22]}
```

Expected: `422` and a message requiring every assigned rubric step. Actual:
`200 {"status":"submitted"}`. The UI disables its submit button until every
box is checked, but the backend does not enforce that invariant.

## Checks completed

| Area | Evidence / result |
| --- | --- |
| Clean install | `npm ci`: pass; 0 reported vulnerabilities. |
| Repository tests | `npm test`: pass — 2 Vitest tests and 3 Rust integration tests. |
| Type/lint | `npx tsc --noEmit`: pass. `cargo clippy --all-targets -- -D warnings`: pass. No JS lint script exists. |
| Production builds | `npm run build`: pass. `cargo build --release --locked`: pass (8.2 MB binary). Docker itself was unavailable in this worker, so image construction could not be run. |
| Core browser flow | Playwright desktop: teacher created two codes and a link; student supplied before/after excerpts and explanation; teacher refreshed queue and marked it reviewed. No browser console errors or page errors. |
| Invalid/recovery paths | Invalid code and empty code selection returned 422; duplicate code 409; reviewed revision re-submit 409; workspace deletion required exact confirmation (422 then 204); deletion removed the student link (404); wrong workspace export was empty. |
| Data isolation/persistence/concurrency | Teacher key export exposed only its own data; student view omitted `student_label`; a record survived server restart; 100 parallel `/api/health` requests all succeeded. |
| Accessibility | Local axe WCAG 2 A/AA scan: **0 violations** (therefore 0 serious/critical). Semantic title/lang/main/h1, labels, alt, and 3px visible focus ring present. At 390px, document scroll width was 390px; reduced-motion transitions computed to `.01ms`. See P3 skip-link note below. |
| Privacy / outbound traffic | Core flow made no third-party requests. Static inspection found only same-origin API calls and the documented Sociobot billing origin. No CDN fonts, analytics, or trackers. `/privacy` and `/terms` render and state no model training, deletion, export, and data minimization. |
| PWA | Service worker installed (`rrl-shell-v1`) and an offline reload showed the app title and h1. Its update cache is not build-versioned; see P2. |
| Performance/bundle | Production JS: 69,785 bytes / 25,920 gzip; CSS: 17,593 / 4,860 gzip; mobile hero: 28,962 bytes. These meet the stated transfer budgets. |
| Live comparison | Live HTML (611 bytes), JS, CSS, and `sw.js` SHA-256 exactly match this candidate's built files. Live invalid-token API behavior also matches. Live `/api/health` returns `{"status":"ok","build_sha":"dev"}`, so backend commit identity remains unproven. |
| Browser/server policy | Live response has CSP, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`, and an `X-Request-Id`. Static JS has no `Cache-Control` header. |

## Defects

### P1 — server accepts incomplete checklist submission

See blocking reproduction above. This is a core workflow/data-integrity failure
and is sufficient for FAIL.

### P2 — deployed backend has no verifiable build identity

`/api/health` returns `build_sha: "dev"` locally and live. The Dockerfile does
not pass a `BUILD_SHA` build argument/environment value. It is therefore not
possible to prove the live backend is `f9606e8`, even though frontend hashes
match exactly.

### P2 — linked rubric deletion returns 500 instead of recoverable conflict

Deleting a rubric that is referenced by a feedback link returned `500
{"error":"Something went wrong while saving. Try again."}`. The intended API
branch and UI copy promise a specific conflict telling the teacher to delete
the link first. The operation is safely rejected, but recovery guidance is
lost.

### P2 — static caching and service-worker update policy do not meet contract

Hashed static JS/CSS are served without `Cache-Control: public, max-age=...,
immutable`. `sw.js` hard-codes `const CACHE = 'rrl-shell-v1'`, rather than a
build-derived cache key, so an update cannot be deterministically identified
or tested as a new cache generation.

### P3 — skip-link destination is not programmatically focusable

Tab exposes the skip link with a 3px ring. Activating it changes the hash and
scrolls to `#main`, but `document.activeElement` becomes `BODY` because main
has no `tabindex="-1"`. The next Tab reaches the first main action, so it is
usable by sighted keyboard users, but assistive technology does not receive a
reliable focused main landmark.

## Required re-verification

Fix server-side checklist equality against the loop's complete rubric-id set,
return the documented linked-rubric conflict, inject the actual Git SHA during
the production build, version service-worker caches per build and add immutable
asset caching. Re-run the complete API and browser workflow and verify the
deployed health SHA before a PASS.
