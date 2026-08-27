<script lang="ts">
  import { onMount } from 'svelte';
  import TeacherWorkspace from './components/TeacherWorkspace.svelte';
  import StudentRevision from './components/StudentRevision.svelte';
  import LegalPage from './components/LegalPage.svelte';

  let online = $state(true);
  const path = window.location.pathname;
  const studentToken = path.startsWith('/r/') ? path.split('/')[2] : null;
  const legal = path === '/privacy' ? 'privacy' : path === '/terms' ? 'terms' : null;

  onMount(() => {
    online = navigator.onLine;
    const update = () => online = navigator.onLine;
    window.addEventListener('online', update);
    window.addEventListener('offline', update);
    return () => { window.removeEventListener('online', update); window.removeEventListener('offline', update); };
  });
</script>

<a class="skip-link" href="#main">Skip to main content</a>
{#if !online}
  <div class="offline-banner" role="status">Offline — saved pages remain readable, but submissions need a connection.</div>
{/if}

{#if studentToken}
  <StudentRevision token={studentToken} />
{:else if legal}
  <LegalPage page={legal} />
{:else}
  <TeacherWorkspace />
{/if}
