import { expect, test } from '@playwright/test';
import fs from 'fs';
import path from 'path';

const repoRoot = path.resolve(__dirname, '../../..');
const screenshotDir = path.join(repoRoot, 'demo-artifacts', 'screenshots');
fs.mkdirSync(screenshotDir, { recursive: true });

function readQuery(): string {
  if (process.env.DEMO_PROMETHEUS_QUERY) return process.env.DEMO_PROMETHEUS_QUERY;
  const queryFile = path.join(repoRoot, 'demo-artifacts', 'review-human', 'logs', 'prometheus-query.txt');
  if (fs.existsSync(queryFile)) return fs.readFileSync(queryFile, 'utf8').trim();
  return 'up';
}

test('Prometheus shows a real query result', async ({ page }) => {
  const query = readQuery();
  const holdMs = Number(process.env.DEMO_BROWSER_SECONDS || '28') * 1000;
  const api = `http://localhost:9090/api/v1/query?query=${encodeURIComponent(query)}`;
  const response = await page.request.get(api);
  expect(response.ok()).toBeTruthy();
  const data = await response.json();
  expect(data?.data?.result?.length || 0).toBeGreaterThan(0);

  const graphUrl = `http://localhost:9090/graph?g0.expr=${encodeURIComponent(query)}&g0.tab=1&g0.show_tree=0`;
  await page.goto(graphUrl, { waitUntil: 'domcontentloaded' });
  await expect(page.locator('body')).toContainText('Prometheus');
  await page.waitForTimeout(1200);
  await page.mouse.move(420, 300);
  await page.waitForTimeout(600);
  await page.mouse.move(1180, 610, { steps: 40 });
  await page.waitForTimeout(4500);
  await page.screenshot({
    path: path.join(screenshotDir, 'prometheus-human-query.png'),
    fullPage: true,
  });
  await page.waitForTimeout(holdMs);
});
