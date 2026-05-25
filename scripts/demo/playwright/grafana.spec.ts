import { expect, test } from '@playwright/test';
import fs from 'fs';
import path from 'path';

const repoRoot = path.resolve(__dirname, '../../..');
const screenshotDir = path.join(repoRoot, 'demo-artifacts', 'screenshots');
fs.mkdirSync(screenshotDir, { recursive: true });

test('Grafana is reachable and can display observability UI', async ({ page }) => {
  await page.goto('http://localhost:3000/login', { waitUntil: 'domcontentloaded' });
  await page.getByLabel(/email or username/i).fill('admin');
  await page.getByPlaceholder('password').fill('admin');
  await page.getByRole('button', { name: /log in/i }).click();
  await page.waitForTimeout(1000);

  if (await page.getByRole('heading', { name: /welcome to grafana/i }).isVisible().catch(() => false)) {
    await page.screenshot({
      path: path.join(screenshotDir, 'grafana-login.png'),
      fullPage: true,
    });
    return;
  }

  await page.goto('http://localhost:3000/dashboards', { waitUntil: 'domcontentloaded' });
  await page.waitForTimeout(1000);
  await page.screenshot({
    path: path.join(screenshotDir, 'grafana-dashboards.png'),
    fullPage: true,
  });

  await page.goto('http://localhost:3000/explore', { waitUntil: 'domcontentloaded' });
  await page.waitForTimeout(1000);
  await expect(page.locator('body')).toContainText(/Explore|Prometheus|Data source/i);
  await page.screenshot({
    path: path.join(screenshotDir, 'grafana-explore.png'),
    fullPage: true,
  });
});
