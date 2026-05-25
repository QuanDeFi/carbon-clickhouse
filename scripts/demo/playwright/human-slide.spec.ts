import { expect, test } from '@playwright/test';
import fs from 'fs';
import path from 'path';

const repoRoot = path.resolve(__dirname, '../../..');
const screenshotDir = path.join(repoRoot, 'demo-artifacts', 'screenshots');
fs.mkdirSync(screenshotDir, { recursive: true });

test('tutorial slide is visible', async ({ page }) => {
  const slide = process.env.DEMO_SLIDE_FILE;
  if (!slide) throw new Error('DEMO_SLIDE_FILE is required');
  const scene = process.env.DEMO_SCENE_ID || path.basename(slide, path.extname(slide));
  const durationMs = Number(process.env.DEMO_SLIDE_SECONDS || '10') * 1000;
  const slidePath = path.isAbsolute(slide) ? slide : path.join(repoRoot, slide);

  await page.goto(`file://${slidePath}`, { waitUntil: 'domcontentloaded' });
  await expect(page.locator('body')).toBeVisible();
  await expect(page.locator('body')).not.toHaveText('');
  await page.mouse.move(280, 280);
  await page.waitForTimeout(500);
  await page.mouse.move(1380, 680, { steps: 28 });
  await page.screenshot({
    path: path.join(screenshotDir, `${scene}.png`),
    fullPage: true,
  });
  await page.waitForTimeout(durationMs);
});
