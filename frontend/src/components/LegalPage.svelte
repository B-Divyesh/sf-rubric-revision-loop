<script lang="ts">
  let { page }: { page: 'privacy' | 'terms' } = $props();
</script>

<header class="simple-header">
  <a class="brand" href="/" aria-label="Rubric Revision Loop home"><span aria-hidden="true">R↻</span> Rubric Revision Loop</a>
</header>
<main id="main" class="legal-page" tabindex="-1">
  {#if page === 'privacy'}
    <p class="eyebrow">Plain-language policy</p>
    <h1>Privacy</h1>
    <p class="lede">Rubric Revision Loop is designed to work without accounts, ads, analytics, or student email addresses.</p>
    <h2>What is stored</h2>
    <p>The service stores rubric guidance, assignment labels, optional student labels, feedback notes, before-and-after excerpts, revision explanations, and review status. A random workspace key in the teacher’s browser controls teacher access. Student links use separate random tokens.</p>
    <h2>How it is used</h2>
    <p>Data is used only to provide the revision workflow. It is not sold, used for advertising, or used to train machine-learning models. No third-party analytics or runtime scripts are loaded.</p>
    <h2>Control and retention</h2>
    <p>Teachers can export or permanently delete their workspace at any time. Free links expire after 30 days; paid retention options can extend a link to 365 days. Teachers should use classroom identifiers rather than full student names.</p>
    <h2>Billing</h2>
    <p>For Studio purchases, Sociobot and Dodo act as merchant of record. This product stores only the license token in the buyer’s browser; payment card details never reach this service.</p>
    <p class="policy-date">Effective August 27, 2026</p>
  {:else}
    <p class="eyebrow">Plain-language policy</p>
    <h1>Terms</h1>
    <p class="lede">Use Rubric Revision Loop to support human-led writing feedback. It does not grade, detect plagiarism, or generate student writing.</p>
    <h2>Teacher responsibility</h2>
    <p>You are responsible for rubric content, review decisions, obtaining any required school consent, and avoiding unnecessary personal data. Do not use the service for emergency, medical, or high-stakes decisions.</p>
    <h2>Availability and acceptable use</h2>
    <p>The service is provided as available. Do not probe, overload, or use it to distribute unlawful or harmful content. Student links should be shared only with their intended recipients.</p>
    <h2>Studio license</h2>
    <p>Studio is a $24 one-time license for the listed features. Sociobot/Dodo is the merchant of record and handles payment and refunds. A refund or chargeback revokes the associated license.</p>
    <h2>Your data</h2>
    <p>You retain responsibility for and ownership of the content you enter. Export is available to every user. See the privacy policy for collection, retention, and deletion details.</p>
    <p class="policy-date">Effective August 27, 2026</p>
  {/if}
  <p><a href="/">Return to your workspace</a></p>
</main>
<footer class="site-footer"><a href="/privacy">Privacy</a><a href="/terms">Terms</a><span>Built for revision, not replacement.</span></footer>
