import { expect, test } from '@playwright/test';
import fs from 'fs';
import path from 'path';

const repoRoot = path.resolve(__dirname, '../../..');
const screenshotDir = path.join(repoRoot, 'demo-artifacts', 'screenshots');
fs.mkdirSync(screenshotDir, { recursive: true });

const queries = [
  'carbon_updates_queued',
  'rate(carbon_updates_processed_total[1m])',
  'rate(carbon_updates_failed_total[1m])',
  'clickhouse_instructions_buffered_rows',
  'rate(clickhouse_instructions_flush_failed_batches[1m])',
  'clickhouse_accounts_buffered_rows',
  'rate(clickhouse_accounts_flush_failed_batches[1m])',
];

test('Prometheus Carbon metrics queries are visible', async ({ page }) => {
  await page.goto('http://localhost:9090', { waitUntil: 'domcontentloaded' });
  await expect(page).toHaveTitle(/Prometheus/i);

  for (const [index, query] of queries.entries()) {
    const url = `http://localhost:9090/graph?g0.expr=${encodeURIComponent(query)}&g0.tab=1&g0.show_tree=0`;
    await page.goto(url, { waitUntil: 'domcontentloaded' });
    await expect(page.locator('body')).toContainText('Prometheus');
    await page.waitForTimeout(500);
    await page.screenshot({
      path: path.join(screenshotDir, `prometheus-${String(index + 1).padStart(2, '0')}.png`),
      fullPage: true,
    });
  }
});
