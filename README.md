# Rubric Revision Loop

Rubric Revision Loop helps teachers reuse criterion-specific feedback while
keeping judgment human. A teacher creates rubric codes, combines them into a
private student link, and reviews the student’s before/after excerpt and
revision explanation in one queue. It does not generate writing, grade work,
or replace an LMS.

Live site: <https://rubric-revision-loop.sociobot.in>

## Who it is for

Teachers handling a large writing-feedback load who want consistent reasons,
specific next steps, and visible revision evidence without sending student
writing to an LLM.

## Product behavior

- The teacher workspace is identified by a random key stored in that browser.
  No account, teacher email, or student email is required.
- Student links reveal only the selected criteria, assignment context, and that
  student’s submission. Free links expire after 30 days.
- Students check each criterion and submit focused before/after excerpts plus
  an explanation. Teachers can review, reopen, copy, or delete each link.
- JSON export and permanent workspace deletion are available to everyone.
- Studio is a $24 one-time unlock for shared rubric-pack links and 90-day or
  one-year retention. Licenses are bought and verified by Sociobot; no payment
  provider is embedded here.

## Local development

Prerequisites: Node 22+, npm, and stable Rust/Cargo.

```bash
npm ci
npm run build          # reproducible frontend build -> dist/index.html
cargo run              # API + dist/ at http://localhost:8080
```

For frontend hot reload, run `cargo run` in one terminal and `npm run dev` in
another. Vite proxies `/api` to port 8080.

## Test and verify

```bash
npm test               # Vitest plus Rust API/integration tests
npm run build
curl http://localhost:8080/api/health
```

Environment variables:

- `PORT` — HTTP port, default `8080`
- `DATABASE_URL` — SQLite URL, default
  `sqlite://data/revision-loop.db?mode=rwc`
- `DIST_DIR` — built frontend path, default `dist`
- `VITE_BILLING_BASE_URL` — optional billing API override for staging builds
- `VITE_PRODUCT_SLUG` is intentionally not needed: the product slug is read
  from the server-owned HTML document metadata.

## Container deployment

```bash
docker build -t rubric-revision-loop .
docker run --rm -p 8080:8080 -v rrl-data:/app/data rubric-revision-loop
```

The multi-stage image runs as a non-root user and serves the Vite build and
Axum API together on `PORT`. Mount `/app/data` as a persistent volume.

## Privacy and design

There are no analytics, ad trackers, third-party fonts, or runtime CDNs.
Student writing is not used for model training. See [`/privacy`](/privacy),
[`/terms`](/terms), [the research brief](.factory/brief.json), and
[the paper-cut visual thesis](.factory/design.md).

Licensed under the MIT License.
