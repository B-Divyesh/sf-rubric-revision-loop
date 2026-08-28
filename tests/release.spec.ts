import { expect, test } from '@playwright/test';
import { resolve } from 'node:path';

const axePath = resolve('node_modules/axe-core/axe.min.js');

test.beforeEach(async ({ context }) => {
  await context.clearCookies();
});

test('teacher, student, and review journey works without browser errors', async ({ browser }) => {
  const context = await browser.newContext({ serviceWorkers: 'block' });
  const teacher = await context.newPage();
  const errors: string[] = [];
  teacher.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  teacher.on('pageerror', error => errors.push(error.message));

  await teacher.goto('/');
  await expect(teacher.getByText('Setting out your workspace…')).toBeHidden();
  await teacher.getByRole('button', { name: /Rubric library/ }).click();
  await teacher.getByRole('button', { name: 'Fill an example' }).click();
  await teacher.getByRole('button', { name: 'Add rubric code' }).click();
  await expect(teacher.getByText('EV-1 added to your rubric library.')).toBeVisible();

  await teacher.getByRole('button', { name: 'Create feedback' }).click();
  await teacher.getByLabel('Assignment title').fill('Argument paragraph');
  await teacher.getByLabel(/EV-1/).check();
  await teacher.getByRole('button', { name: 'Create student link' }).click();
  const revisionLink = await teacher.getByLabel('Student revision link').first().inputValue();

  const student = await context.newPage();
  student.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  student.on('pageerror', error => errors.push(error.message));
  await student.goto(revisionLink);
  await student.getByRole('checkbox', { name: /Explain evidence/ }).check();
  await student.getByLabel('Before excerpt').fill('The quotation proves the claim.');
  await student.getByLabel('After excerpt').fill('The quotation supports the claim because its detail shows the cost increased.');
  await student.getByLabel('Revision explanation').fill('I explained how the quoted detail supports my claim.');
  await student.getByRole('button', { name: 'Send revision to teacher' }).click();
  await expect(student.getByText('Revision sent. Your teacher can now review the evidence.')).toBeFocused();

  await teacher.getByRole('button', { name: /Review queue/ }).click();
  await teacher.getByRole('button', { name: 'Refresh queue' }).click();
  await expect(teacher.getByText('Ready to review', { exact: true })).toBeVisible();
  await teacher.getByRole('button', { name: 'Mark reviewed' }).click();
  await student.reload();
  await expect(student.getByText('Reviewed by your teacher')).toBeVisible();
  await expect(student.getByLabel('Before excerpt')).toBeDisabled();
  expect(errors).toEqual([]);
  await context.close();
});

test('390px workspace keeps its h1 and all persistent navigation targets accessible', async ({ browser }) => {
  const context = await browser.newContext({
    viewport: { width: 390, height: 844 },
    reducedMotion: 'reduce',
    serviceWorkers: 'block',
    bypassCSP: true
  });
  const page = await context.newPage();
  const errors: string[] = [];
  page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('/');
  await expect(page.getByText('Setting out your workspace…')).toBeHidden();

  const h1 = page.getByRole('heading', { level: 1 });
  await expect(h1).toHaveCount(1);
  await expect(h1).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);

  for (const link of [
    page.getByRole('link', { name: 'Rubric Revision Loop home' }),
    page.getByRole('link', { name: 'Privacy' }),
    page.getByRole('link', { name: 'Terms' })
  ]) {
    const box = await link.boundingBox();
    expect(box, 'target must have a layout box').not.toBeNull();
    expect(box!.height).toBeGreaterThanOrEqual(44);
    expect(box!.width).toBeGreaterThanOrEqual(44);
  }

  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: 'Skip to main content' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('#main')).toBeFocused();
  await page.addScriptTag({ path: axePath });
  const violations = await page.evaluate(async () => {
    const result = await (window as unknown as { axe: { run: (options: unknown) => Promise<{ violations: { impact: string | null }[] }> } }).axe.run({
      runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa', 'wcag21aa'] }
    });
    return result.violations.filter(item => item.impact === 'serious' || item.impact === 'critical');
  });
  expect(violations).toEqual([]);
  expect(errors).toEqual([]);
  await context.close();
});

test('service worker keeps an explicit offline shell', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  await page.goto('/');
  await page.evaluate(() => navigator.serviceWorker.ready);
  await page.reload();
  await expect.poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller))).toBe(true);
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
  await expect(page.getByRole('status')).toContainText('Offline');
  await context.close();
});

test('Studio checkout return, restore, and revocation follow the billing contract', async ({ page }) => {
  await page.route('https://api.sociobot.in/api/v1/products/rubric-revision-loop/verify**', async route => {
    const token = new URL(route.request().url()).searchParams.get('license');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ valid: token !== 'revoked-token', reason: token === 'revoked-token' ? 'revoked' : 'ok' })
    });
  });
  await page.goto('/?license=issued-return-token');
  await expect(page.getByText('Setting out your workspace…')).toBeHidden();
  await expect(page).toHaveURL('/');
  expect(await page.evaluate(() => localStorage.getItem('sb_license:rubric-revision-loop'))).toBe('issued-return-token');
  await page.getByRole('button', { name: 'Workspace' }).click();
  await expect(page.getByText('Unlocked', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Remove license from this browser' }).click();
  await expect(page.getByRole('link', { name: 'Buy Studio — $24 once' })).toHaveAttribute(
    'href',
    'https://api.sociobot.in/api/v1/products/rubric-revision-loop/checkout'
  );
  const restore = page.getByLabel(/Have a license/);
  await restore.fill('revoked-token');
  await page.getByRole('button', { name: 'Verify license' }).click();
  await expect(page.getByText('License no longer active.')).toBeVisible();
  await restore.fill('restored-valid-token');
  await page.getByRole('button', { name: 'Verify license' }).click();
  await expect(page.getByText('Unlocked', { exact: true })).toBeVisible();
});
