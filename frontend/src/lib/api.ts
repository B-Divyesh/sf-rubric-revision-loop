export type Rubric = { id: number; code: string; title: string; guidance: string; next_step: string; created_at: string };
export type FeedbackLoop = {
  id: number; token: string; student_label: string; assignment_title: string; teacher_note: string;
  status: 'awaiting' | 'submitted' | 'reviewed'; before_excerpt: string | null; after_excerpt: string | null;
  explanation: string | null; checklist: number[]; retention_days: number; created_at: string;
  submitted_at: string | null; reviewed_at: string | null; rubrics: Rubric[];
};
export type StudentLoop = Omit<FeedbackLoop, 'id' | 'token' | 'student_label' | 'created_at' | 'retention_days' | 'submitted_at' | 'reviewed_at'> & { expires_at: string };

const KEY_NAME = 'rrl_workspace_key';

export function getWorkspaceKey(): string {
  let key = localStorage.getItem(KEY_NAME);
  if (!key) {
    key = crypto.randomUUID().replaceAll('-', '') + crypto.randomUUID().replaceAll('-', '');
    localStorage.setItem(KEY_NAME, key);
  }
  return key;
}

export async function api<T>(path: string, options: RequestInit = {}, teacher = true): Promise<T> {
  const headers = new Headers(options.headers);
  if (teacher) headers.set('x-workspace-key', getWorkspaceKey());
  if (options.body) headers.set('content-type', 'application/json');
  let response: Response;
  try { response = await fetch(`/api${path}`, { ...options, headers }); }
  catch { throw new Error('You appear to be offline. Reconnect and try again.'); }
  if (!response.ok) {
    const data = await response.json().catch(() => ({}));
    throw new Error(data.error || `The request failed (${response.status}). Try again.`);
  }
  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
}

export function revisionUrl(token: string): string {
  return `${window.location.origin}/r/${token}`;
}

export function formatDate(value: string | null): string {
  if (!value) return 'Not yet';
  const normalized = value.includes('T') ? value : `${value.replace(' ', 'T')}Z`;
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(normalized));
}

export function slugFromDocument(): string {
  return document.documentElement.dataset.productSlug || 'rubric-revision-loop';
}
