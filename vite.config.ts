import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const buildSha = (process.env.BUILD_SHA || 'dev').replace(/[^a-zA-Z0-9_-]/g, '');

function versionServiceWorker() {
  return {
    name: 'version-service-worker-cache',
    writeBundle(options: { dir?: string }) {
      const worker = resolve(options.dir || 'dist', 'sw.js');
      const template = readFileSync(worker, 'utf8');
      writeFileSync(worker, template.replaceAll('__BUILD_SHA__', buildSha));
    }
  };
}

export default defineConfig({
  root: 'frontend',
  plugins: [svelte(), versionServiceWorker()],
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    target: 'es2022',
    sourcemap: false
  },
  server: {
    proxy: { '/api': 'http://localhost:8080' }
  },
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node'
  }
});
