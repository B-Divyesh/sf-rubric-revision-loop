import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: 1,
  reporter: 'line',
  use: {
    baseURL: 'http://127.0.0.1:4173',
    browserName: 'chromium',
    trace: 'retain-on-failure'
  },
  webServer: {
    command: 'cargo run --quiet',
    url: 'http://127.0.0.1:4173/api/health',
    reuseExistingServer: false,
    timeout: 120_000,
    env: {
      ...process.env,
      PORT: '4173',
      DATABASE_URL: 'sqlite://target/playwright-release.db?mode=rwc',
      DIST_DIR: 'dist'
    }
  }
});
