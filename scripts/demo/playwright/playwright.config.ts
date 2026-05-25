import { defineConfig } from '@playwright/test';
import path from 'path';

const repoRoot = path.resolve(__dirname, '../../..');

export default defineConfig({
  testDir: __dirname,
  timeout: 60_000,
  use: {
    headless: false,
    viewport: { width: 1920, height: 1080 },
    launchOptions: {
      args: [
        '--disable-gpu',
        '--disable-dev-shm-usage',
        '--no-sandbox',
        '--window-position=0,0',
        '--window-size=1920,1080',
      ],
    },
    screenshot: 'only-on-failure',
    video: 'off',
  },
  outputDir: path.join(repoRoot, 'demo-artifacts', 'playwright-results'),
});
