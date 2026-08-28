<script lang="ts">
  import { onMount } from 'svelte';
  import { api, formatDate, type StudentLoop } from '../lib/api';
  let { token }: { token: string } = $props();

  let item = $state<StudentLoop | null>(null);
  let loading = $state(true);
  let saving = $state(false);
  let error = $state('');
  let success = $state('');
  let before = $state('');
  let after = $state('');
  let explanation = $state('');
  let checklist = $state<number[]>([]);

  onMount(load);
  async function load() {
    loading = true; error = '';
    try {
      item = await api<StudentLoop>(`/student/${token}`, {}, false);
      before = item.before_excerpt || ''; after = item.after_excerpt || '';
      explanation = item.explanation || ''; checklist = item.checklist || [];
    } catch (e) { error = (e as Error).message; }
    finally { loading = false; }
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault(); saving = true; error = ''; success = '';
    try {
      await api(`/student/${token}/revision`, { method: 'POST', body: JSON.stringify({ before_excerpt: before, after_excerpt: after, explanation, checklist }) }, false);
      success = 'Revision sent. Your teacher can now review the evidence.';
      await load();
      document.getElementById('student-status')?.focus();
    } catch (e) { error = (e as Error).message; }
    finally { saving = false; }
  }
</script>

<header class="student-header">
  <a class="brand" href="/" aria-label="Rubric Revision Loop home"><span aria-hidden="true">R↻</span> Revision slip</a>
  <span class="privacy-chip">No account needed</span>
</header>
<main id="main" class="student-main" tabindex="-1">
  {#if loading}
    <div class="loading-state" aria-live="polite"><span class="paper-pulse" aria-hidden="true"></span><p>Opening your revision slip…</p></div>
  {:else if error && !item}
    <section class="state-sheet error-sheet">
      <p class="eyebrow">Link unavailable</p><h1>We couldn’t open this revision.</h1><p>{error}</p>
      <button class="button secondary" onclick={load}>Try again</button>
    </section>
  {:else if item}
    <section class="student-intro">
      <p class="eyebrow">Revision for</p>
      <h1>{item.assignment_title}</h1>
      <p class="lede">Use your teacher’s reasons, show the exact change, then explain what the change accomplishes.</p>
      {#if item.teacher_note}<aside class="teacher-note"><strong>Teacher note</strong><p>{item.teacher_note}</p></aside>{/if}
      <p class="expiry">Link available until {formatDate(item.expires_at)}.</p>
    </section>

    <form class="revision-form" onsubmit={submit}>
      <section aria-labelledby="steps-title">
        <div class="section-heading"><span class="step-number">1</span><div><h2 id="steps-title">Read your revision steps</h2><p>Check each one when your revision addresses it.</p></div></div>
        <div class="rubric-stack student-rubrics">
          {#each item.rubrics as rubric}
            <label class="rubric-check">
              <input type="checkbox" bind:group={checklist} value={rubric.id} disabled={item.status === 'reviewed'} />
              <span class="rubric-tab">{rubric.code}</span>
              <span><strong>{rubric.title}</strong><span>{rubric.guidance}</span><em>Try next: {rubric.next_step}</em></span>
            </label>
          {/each}
        </div>
      </section>

      <section aria-labelledby="evidence-title">
        <div class="section-heading"><span class="step-number">2</span><div><h2 id="evidence-title">Show the change</h2><p>Paste only the relevant sentences, not the whole assignment.</p></div></div>
        <div class="before-after">
          <label>Before excerpt<textarea bind:value={before} required maxlength="4000" rows="7" disabled={item.status === 'reviewed'}></textarea><small>{before.length}/4000</small></label>
          <div class="paper-arrow" aria-hidden="true">→</div>
          <label>After excerpt<textarea bind:value={after} required maxlength="4000" rows="7" disabled={item.status === 'reviewed'}></textarea><small>{after.length}/4000</small></label>
        </div>
      </section>

      <section aria-labelledby="explain-title">
        <div class="section-heading"><span class="step-number">3</span><div><h2 id="explain-title">Explain your decision</h2><p>What did you change, and how does it improve the writing?</p></div></div>
        <label class="sr-only" for="explanation">Revision explanation</label>
        <textarea id="explanation" bind:value={explanation} required minlength="8" maxlength="2000" rows="5" disabled={item.status === 'reviewed'}></textarea>
      </section>

      <div id="student-status" class:success-message={success} class:error-message={error} role="status" tabindex="-1">{success || error}</div>
      {#if item.status === 'reviewed'}
        <div class="reviewed-seal"><span aria-hidden="true">✓</span><strong>Reviewed by your teacher</strong><p>This revision slip is now read-only.</p></div>
      {:else}
        <button class="button primary submit-revision" disabled={saving || checklist.length !== item.rubrics.length}>{saving ? 'Sending revision…' : item.status === 'submitted' ? 'Update revision evidence' : 'Send revision to teacher'}</button>
        {#if checklist.length !== item.rubrics.length}<p class="form-hint">Check every revision step before sending.</p>{/if}
      {/if}
    </form>
  {/if}
</main>
<footer class="site-footer"><a href="/privacy">Privacy</a><a href="/terms">Terms</a><span>Only your teacher decides what counts as progress.</span></footer>
