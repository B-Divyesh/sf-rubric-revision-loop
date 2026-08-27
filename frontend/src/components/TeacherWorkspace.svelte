<script lang="ts">
  import { onMount } from 'svelte';
  import { api, formatDate, getWorkspaceKey, revisionUrl, type FeedbackLoop, type Rubric } from '../lib/api';
  import { captureLicense, checkoutUrl, clearLicense, saveLicense, verifyLicense, type LicenseState } from '../lib/license';

  type Tab = 'create' | 'rubrics' | 'queue' | 'settings';
  let tab = $state<Tab>('create');
  let rubrics = $state<Rubric[]>([]);
  let loops = $state<FeedbackLoop[]>([]);
  let loading = $state(true);
  let busy = $state(false);
  let notice = $state('');
  let error = $state('');
  let createdUrl = $state('');
  let copied = $state('');
  let licenseState = $state<LicenseState>('checking');
  let licenseInput = $state('');
  let deletePhrase = $state('');
  let packUrl = $state('');
  let packToken = $state('');
  let incomingPack = $state('');

  let code = $state(''); let title = $state(''); let guidance = $state(''); let nextStep = $state('');
  let assignmentTitle = $state(''); let studentLabel = $state(''); let teacherNote = $state('');
  let selectedIds = $state<number[]>([]); let retentionDays = $state(30);

  onMount(async () => {
    captureLicense();
    const queryPack = new URL(window.location.href).searchParams.get('pack');
    if (queryPack) { incomingPack = queryPack; packToken = queryPack; tab = 'settings'; }
    licenseState = await verifyLicense();
    await reload();
  });

  async function reload() {
    loading = true; error = '';
    try {
      const [rubricData, loopData] = await Promise.all([
        api<{ items: Rubric[] }>('/rubrics'), api<{ items: FeedbackLoop[] }>('/loops')
      ]);
      rubrics = rubricData.items; loops = loopData.items;
    } catch (e) { error = (e as Error).message; }
    finally { loading = false; }
  }

  function announce(message: string) { notice = message; window.setTimeout(() => { if (notice === message) notice = ''; }, 5000); }

  async function addRubric(event: SubmitEvent) {
    event.preventDefault(); busy = true; error = '';
    try {
      const added = await api<Rubric>('/rubrics', { method: 'POST', body: JSON.stringify({ code, title, guidance, next_step: nextStep }) });
      rubrics = [...rubrics, added].sort((a, b) => a.code.localeCompare(b.code));
      code = title = guidance = nextStep = ''; announce(`${added.code} added to your rubric library.`);
    } catch (e) { error = (e as Error).message; }
    finally { busy = false; }
  }

  function fillExample() {
    code = 'EV-1'; title = 'Explain evidence';
    guidance = 'The evidence is present, but the connection to your claim needs to be explicit.';
    nextStep = 'Add one sentence that explains how the quoted detail supports your claim.';
  }

  async function removeRubric(rubric: Rubric) {
    if (!confirm(`Delete ${rubric.code} — ${rubric.title}? Existing feedback links must be deleted first.`)) return;
    try { await api(`/rubrics/${rubric.id}`, { method: 'DELETE' }); rubrics = rubrics.filter(r => r.id !== rubric.id); announce(`${rubric.code} deleted.`); }
    catch (e) { error = (e as Error).message; }
  }

  async function createLink(event: SubmitEvent) {
    event.preventDefault(); busy = true; error = ''; createdUrl = '';
    try {
      const created = await api<{ token: string }>('/loops', { method: 'POST', body: JSON.stringify({ assignment_title: assignmentTitle, student_label: studentLabel, teacher_note: teacherNote, rubric_ids: selectedIds, retention_days: licenseState === 'unlocked' ? retentionDays : 30 }) });
      createdUrl = revisionUrl(created.token); assignmentTitle = studentLabel = teacherNote = ''; selectedIds = []; retentionDays = 30;
      announce('Student revision link created.'); await reload();
    } catch (e) { error = (e as Error).message; }
    finally { busy = false; }
  }

  async function copyText(value: string, name: string) {
    try { await navigator.clipboard.writeText(value); copied = name; announce('Link copied to clipboard.'); window.setTimeout(() => copied = '', 2000); }
    catch { error = 'Your browser blocked clipboard access. Select and copy the link manually.'; }
  }

  async function setReviewed(loop: FeedbackLoop, reviewed: boolean) {
    try { await api(`/loops/${loop.id}/review`, { method: 'PATCH', body: JSON.stringify({ reviewed }) }); await reload(); announce(reviewed ? 'Revision marked reviewed.' : 'Revision reopened.'); }
    catch (e) { error = (e as Error).message; }
  }

  async function removeLoop(loop: FeedbackLoop) {
    const label = loop.student_label || loop.assignment_title;
    if (!confirm(`Permanently delete the feedback link for “${label}”? The student will no longer be able to open it.`)) return;
    try { await api(`/loops/${loop.id}`, { method: 'DELETE' }); await reload(); announce('Feedback link deleted.'); }
    catch (e) { error = (e as Error).message; }
  }

  async function exportData() {
    error = '';
    try {
      const response = await fetch('/api/export', { headers: { 'x-workspace-key': getWorkspaceKey() } });
      if (!response.ok) throw new Error('The export could not be prepared. Try again.');
      const href = URL.createObjectURL(await response.blob()); const link = document.createElement('a');
      link.href = href; link.download = 'revision-loop-export.json'; link.click(); URL.revokeObjectURL(href); announce('Workspace export downloaded.');
    } catch (e) { error = (e as Error).message; }
  }

  async function deleteWorkspace() {
    if (deletePhrase !== 'delete my workspace') return;
    if (!confirm('Delete every rubric, feedback link, and revision in this workspace? This cannot be undone.')) return;
    try {
      await api('/workspace', { method: 'DELETE', headers: { 'x-confirm-delete': deletePhrase } });
      localStorage.removeItem('rrl_workspace_key'); rubrics = []; loops = []; deletePhrase = ''; tab = 'rubrics'; announce('Workspace data permanently deleted. A fresh workspace is ready.');
    } catch (e) { error = (e as Error).message; }
  }

  async function restoreLicense(event: SubmitEvent) {
    event.preventDefault(); if (!licenseInput.trim()) return; saveLicense(licenseInput); licenseState = 'checking'; licenseState = await verifyLicense(true);
    if (licenseState === 'unlocked') { licenseInput = ''; announce('Studio license restored.'); } else if (licenseState === 'invalid') error = 'That license is not active. Check the token and try again.'; else error = 'License verification is unavailable. Reconnect and try again.';
  }

  function removeLicense() { clearLicense(); licenseState = 'locked'; announce('License removed from this browser.'); }

  async function sharePack() {
    if (licenseState !== 'unlocked') return;
    try { const result = await api<{ token: string }>('/packs', { method: 'POST', body: JSON.stringify({ rubric_ids: rubrics.map(r => r.id) }) }); packUrl = `${location.origin}/?pack=${result.token}`; announce('Team rubric pack link created.'); }
    catch (e) { error = (e as Error).message; }
  }

  async function importPack() {
    if (licenseState !== 'unlocked') return;
    const token = packToken.trim(); if (!token) return;
    try { const result = await api<{ imported: number }>(`/packs/${encodeURIComponent(token)}/import`, { method: 'POST' }); await reload(); announce(`${result.imported} rubric code${result.imported === 1 ? '' : 's'} imported.`); incomingPack = ''; packToken = ''; }
    catch (e) { error = (e as Error).message; }
  }

  const submittedCount = $derived(loops.filter(l => l.status === 'submitted').length);
</script>

<header class="app-header">
  <a class="brand" href="/" aria-label="Rubric Revision Loop home"><span aria-hidden="true">R↻</span> Rubric Revision Loop</a>
  <span class="local-chip"><span aria-hidden="true">●</span> Private workspace</span>
</header>

<div class="app-shell">
  <aside class="sidebar">
    <div class="sidebar-title"><p class="eyebrow">Teacher desk</p><h1>Make the next revision visible.</h1></div>
    <nav aria-label="Workspace">
      <button class:active={tab === 'create'} aria-current={tab === 'create' ? 'page' : undefined} onclick={() => tab = 'create'}><span aria-hidden="true">↗</span> Create feedback</button>
      <button class:active={tab === 'rubrics'} aria-current={tab === 'rubrics' ? 'page' : undefined} onclick={() => tab = 'rubrics'}><span aria-hidden="true">▤</span> Rubric library <b>{rubrics.length}</b></button>
      <button class:active={tab === 'queue'} aria-current={tab === 'queue' ? 'page' : undefined} onclick={() => tab = 'queue'}><span aria-hidden="true">≡</span> Review queue {#if submittedCount}<b class="alert-count">{submittedCount}</b>{/if}</button>
      <button class:active={tab === 'settings'} aria-current={tab === 'settings' ? 'page' : undefined} onclick={() => tab = 'settings'}><span aria-hidden="true">⚙</span> Workspace</button>
    </nav>
    <div class="principle-note"><span aria-hidden="true">✎</span><p><strong>Your judgment stays central.</strong><br />No generated feedback. No automated grades.</p></div>
  </aside>

  <main id="main" class="workspace-main">
    <div class="live-region" aria-live="polite">{notice}</div>
    {#if error}<div class="error-banner" role="alert"><span>{error}</span><button aria-label="Dismiss error" onclick={() => error = ''}>×</button></div>{/if}
    {#if loading}
      <div class="loading-state" aria-live="polite"><span class="paper-pulse" aria-hidden="true"></span><p>Setting out your workspace…</p></div>
    {:else if tab === 'create'}
      <section class="page-heading"><div><p class="eyebrow">New feedback loop</p><h2>Turn a rubric reason into a revision</h2><p>Choose reusable codes, add only the context this student needs, then share one focused link.</p></div><span class="queue-stamp"><strong>{submittedCount}</strong> ready to review</span></section>
      {#if rubrics.length === 0}
        <section class="empty-hero">
          <picture><source media="(max-width: 700px)" srcset="/assets/revision-loop-hero-720.webp" /><img src="/assets/revision-loop-hero.webp" width="1200" height="800" alt="A paper-cut path connects an annotated page to before-and-after excerpts and a review tray." fetchpriority="high" /></picture>
          <div><p class="eyebrow">Start with one reason you repeat</p><h2>Your rubric library is empty</h2><p>Create a short code for feedback you give often. Students will see both your explanation and the concrete next step.</p><button class="button primary" onclick={() => tab = 'rubrics'}>Create your first code</button></div>
        </section>
      {:else}
        <form class="paper-form link-form" onsubmit={createLink}>
          <div class="form-grid">
            <label>Assignment title <input bind:value={assignmentTitle} required minlength="2" maxlength="120" autocomplete="off" /></label>
            <label>Student label <span class="optional">Optional — use initials or class ID</span><input bind:value={studentLabel} maxlength="80" autocomplete="off" /></label>
          </div>
          <fieldset><legend>Choose rubric codes <span>1–12</span></legend><div class="code-picker">{#each rubrics as rubric}<label class:selected={selectedIds.includes(rubric.id)}><input type="checkbox" bind:group={selectedIds} value={rubric.id} /><span><b>{rubric.code}</b>{rubric.title}</span></label>{/each}</div></fieldset>
          <label>Personal note <span class="optional">Optional context—don’t repeat the rubric</span><textarea bind:value={teacherNote} maxlength="800" rows="4"></textarea></label>
          <div class="retention-row"><div><strong>Student link retention</strong><p>{licenseState === 'unlocked' ? 'Choose how long this link remains available.' : 'Free links expire after 30 days.'}</p></div>{#if licenseState === 'unlocked'}<label><span class="sr-only">Retention duration</span><select bind:value={retentionDays}><option value={30}>30 days</option><option value={90}>90 days</option><option value={365}>1 year</option></select></label>{:else}<button type="button" class="text-button" onclick={() => tab = 'settings'}>See Studio options</button>{/if}</div>
          <button class="button primary" disabled={busy || selectedIds.length === 0}>{busy ? 'Creating link…' : 'Create student link'}</button>
        </form>
        {#if createdUrl}<section class="created-slip"><p class="eyebrow">Ready to share</p><h3>Student revision link</h3><p>This link contains the selected guidance, not the rest of your workspace.</p><div class="copy-row"><input aria-label="Student revision link" readonly value={createdUrl} onclick={(e) => (e.currentTarget as HTMLInputElement).select()} /><button class="button primary" onclick={() => copyText(createdUrl, 'created')}>{copied === 'created' ? 'Copied' : 'Copy link'}</button></div></section>{/if}
      {/if}
    {:else if tab === 'rubrics'}
      <section class="page-heading"><div><p class="eyebrow">Reusable reasons</p><h2>Rubric code library</h2><p>Write the explanation once, then keep your judgment specific to each piece.</p></div></section>
      <form class="paper-form rubric-form" onsubmit={addRubric}>
        <div class="form-grid"><label>Short code <input bind:value={code} required minlength="2" maxlength="12" pattern="[A-Za-z0-9.\-]+" placeholder="EV-1" /></label><label>Criterion name <input bind:value={title} required minlength="2" maxlength="80" placeholder="Explain evidence" /></label></div>
        <label>What the student should understand <textarea bind:value={guidance} required minlength="8" maxlength="600" rows="3"></textarea></label>
        <label>Concrete revision prompt <textarea bind:value={nextStep} required minlength="8" maxlength="300" rows="2"></textarea></label>
        <div class="form-actions"><button class="button primary" disabled={busy}>{busy ? 'Adding code…' : 'Add rubric code'}</button>{#if rubrics.length === 0}<button class="button secondary" type="button" onclick={fillExample}>Fill an example</button>{/if}</div>
      </form>
      {#if rubrics.length === 0}<div class="empty-inline"><strong>No codes yet.</strong><p>Start with the feedback sentence you write most often.</p></div>{:else}<div class="rubric-library">{#each rubrics as rubric}<article class="rubric-sheet"><div class="rubric-sheet-head"><span class="rubric-tab">{rubric.code}</span><div><h3>{rubric.title}</h3><p>{rubric.guidance}</p></div></div><div class="next-step"><span aria-hidden="true">↳</span><div><strong>Revision prompt</strong><p>{rubric.next_step}</p></div></div><button class="text-button danger" onclick={() => removeRubric(rubric)}>Delete {rubric.code}</button></article>{/each}</div>{/if}
    {:else if tab === 'queue'}
      <section class="page-heading"><div><p class="eyebrow">Returned work</p><h2>Review queue</h2><p>Compare the exact excerpt, read the student’s reasoning, and close the loop.</p></div><button class="button secondary" onclick={reload}>Refresh queue</button></section>
      <div class="queue-filters" aria-label="Queue summary"><span><b>{loops.filter(l => l.status === 'submitted').length}</b> Ready</span><span><b>{loops.filter(l => l.status === 'awaiting').length}</b> Awaiting</span><span><b>{loops.filter(l => l.status === 'reviewed').length}</b> Reviewed</span></div>
      {#if loops.length === 0}<section class="empty-inline"><strong>Your tray is clear.</strong><p>Create a feedback link, then student revisions will arrive here.</p><button class="button primary" onclick={() => tab = 'create'}>Create feedback</button></section>{:else}<div class="queue-list">{#each loops as loop}<details class="queue-sheet" open={loop.status === 'submitted'}><summary><span class:submitted={loop.status === 'submitted'} class:reviewed={loop.status === 'reviewed'} class="status-mark">{loop.status === 'submitted' ? 'Ready to review' : loop.status === 'reviewed' ? 'Reviewed' : 'Awaiting student'}</span><span class="queue-name"><strong>{loop.student_label || 'Unlabelled student'}</strong><span>{loop.assignment_title}</span></span><span class="queue-date">{formatDate(loop.submitted_at || loop.created_at)}</span></summary><div class="queue-body"><div class="assigned-codes">{#each loop.rubrics as rubric}<span>{rubric.code}</span>{/each}</div>{#if loop.status === 'awaiting'}<p class="awaiting-note">No revision submitted yet. Share the link again if needed.</p>{:else}<div class="comparison"><section><h3>Before</h3><blockquote>{loop.before_excerpt}</blockquote></section><section><h3>After</h3><blockquote>{loop.after_excerpt}</blockquote></section></div><div class="explanation-sheet"><h3>Student’s explanation</h3><p>{loop.explanation}</p></div>{/if}<div class="copy-row compact"><input aria-label="Student revision link" readonly value={revisionUrl(loop.token)} /><button class="button secondary" onclick={() => copyText(revisionUrl(loop.token), String(loop.id))}>{copied === String(loop.id) ? 'Copied' : 'Copy link'}</button></div><div class="review-actions">{#if loop.status === 'submitted'}<button class="button primary" onclick={() => setReviewed(loop, true)}>Mark reviewed</button>{:else if loop.status === 'reviewed'}<button class="button secondary" onclick={() => setReviewed(loop, false)}>Reopen revision</button>{/if}<button class="text-button danger" onclick={() => removeLoop(loop)}>Delete link</button></div></div></details>{/each}</div>{/if}
    {:else}
      <section class="page-heading"><div><p class="eyebrow">Data and team tools</p><h2>Workspace</h2><p>Export anything you store, set retention, or unlock shared rubric packs.</p></div></section>
      <section class="settings-section"><h3>Your data</h3><p>This browser holds a private workspace key. Keep an export before clearing browser storage. Student names and emails are not required.</p><button class="button secondary" onclick={exportData}>Export all data</button></section>
      <section class="settings-section studio-section"><div class="studio-title"><div><span class="rubric-tab gold">Studio</span><h3>Shared rubrics and longer retention</h3></div><span class:unlocked={licenseState === 'unlocked'} class="license-status">{licenseState === 'unlocked' ? 'Unlocked' : licenseState === 'checking' ? 'Checking…' : 'Locked'}</span></div><p><strong>$24 one-time.</strong> Create team rubric-pack links and retain student links for 90 days or one year. The free workflow, accessibility, and export remain available.</p>{#if licenseState === 'unlocked'}<div class="studio-tools"><h4>Share this rubric library</h4><p>Anyone with the pack link and a Studio license can copy these codes into their workspace.</p><button class="button primary" onclick={sharePack} disabled={rubrics.length === 0}>Create team pack link</button>{#if packUrl}<div class="copy-row"><input aria-label="Team rubric pack link" readonly value={packUrl} /><button class="button secondary" onclick={() => copyText(packUrl, 'pack')}>{copied === 'pack' ? 'Copied' : 'Copy link'}</button></div>{/if}<h4>Import a team pack</h4><div class="copy-row"><input aria-label="Team pack code" bind:value={packToken} placeholder="Paste pack code or open a pack link" /><button class="button secondary" onclick={importPack} disabled={!packToken}>Import codes</button></div><button class="text-button" onclick={removeLicense}>Remove license from this browser</button></div>{:else}<a class="button primary" href={checkoutUrl()}>Buy Studio — $24 once</a><form class="restore-form" onsubmit={restoreLicense}><label>Have a license? <span>Paste it to restore this purchase.</span><input bind:value={licenseInput} autocomplete="off" /></label><button class="button secondary">Verify license</button></form>{#if licenseState === 'invalid'}<p class="quiet-warning">License no longer active. You can continue using the free workspace or purchase a new license.</p>{/if}<p class="merchant-note">Secure checkout and refunds are handled by Sociobot/Dodo, the merchant of record.</p>{/if}</section>
      <section class="settings-section danger-zone"><h3>Delete workspace</h3><p>Permanently removes all rubric codes, links, and student revision evidence associated with this browser’s key.</p><label>Type <strong>delete my workspace</strong> to confirm<input bind:value={deletePhrase} autocomplete="off" /></label><button class="button danger-button" disabled={deletePhrase !== 'delete my workspace'} onclick={deleteWorkspace}>Delete all workspace data</button></section>
    {/if}
  </main>
</div>
<footer class="site-footer app-footer"><a href="/privacy">Privacy</a><a href="/terms">Terms</a><span>Original paper illustration generated for this product.</span></footer>
