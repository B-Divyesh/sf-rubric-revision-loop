import { slugFromDocument } from './api';

export type LicenseState = 'checking' | 'unlocked' | 'locked' | 'invalid';
type Verdict = { valid: boolean; checked_at: number };

const DAY = 86_400_000;

export function licenseKey(): string { return `sb_license:${slugFromDocument()}`; }
export function verdictKey(): string { return `sb_license_verdict:${slugFromDocument()}`; }
export function checkoutUrl(): string { return `https://api.sociobot.in/api/v1/products/${slugFromDocument()}/checkout`; }

export function captureLicense(): string | null {
  const url = new URL(window.location.href);
  const incoming = url.searchParams.get('license');
  if (incoming) {
    localStorage.setItem(licenseKey(), incoming.trim());
    url.searchParams.delete('license');
    history.replaceState({}, '', url.pathname + url.search + url.hash);
  }
  return incoming;
}

export async function verifyLicense(force = false): Promise<LicenseState> {
  const token = localStorage.getItem(licenseKey());
  if (!token) return 'locked';
  const cached = readVerdict();
  if (!force && cached && Date.now() - cached.checked_at < DAY) return cached.valid ? 'unlocked' : 'invalid';
  try {
    const base = import.meta.env.VITE_BILLING_BASE_URL || 'https://api.sociobot.in';
    const response = await fetch(`${base}/api/v1/products/${slugFromDocument()}/verify?license=${encodeURIComponent(token)}`);
    if (!response.ok) throw new Error('verification unavailable');
    const result = await response.json() as { valid: boolean };
    localStorage.setItem(verdictKey(), JSON.stringify({ valid: result.valid, checked_at: Date.now() }));
    return result.valid ? 'unlocked' : 'invalid';
  } catch {
    return cached?.valid ? 'unlocked' : 'checking';
  }
}

export function saveLicense(token: string): void {
  localStorage.setItem(licenseKey(), token.trim());
  localStorage.removeItem(verdictKey());
}

export function clearLicense(): void {
  localStorage.removeItem(licenseKey());
  localStorage.removeItem(verdictKey());
}

function readVerdict(): Verdict | null {
  try { return JSON.parse(localStorage.getItem(verdictKey()) || 'null') as Verdict | null; }
  catch { return null; }
}
