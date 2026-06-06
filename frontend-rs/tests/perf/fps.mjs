// FPS measurement - frames per second during canvas interaction
// Usage: node tests/perf/fps.mjs

import { chromium } from 'playwright';

const browser = await chromium.launch();
const page = await browser.newPage();
await page.goto('http://localhost:8080', { waitUntil: 'domcontentloaded' });
await page.waitForSelector('[data-testid=editor-ready]', { timeout: 30000 });

const frameTimes = await page.evaluate(() => {
  const times = [];
  let lastTime = performance.now();
  const measure = () => {
    const now = performance.now();
    times.push(now - lastTime);
    lastTime = now;
    if (times.length < 100) requestAnimationFrame(measure);
  };
  requestAnimationFrame(measure);
  return new Promise(resolve => setTimeout(() => resolve(times), 2000));
});

await browser.close();
const sorted = frameTimes.sort((a, b) => a - b);
const p50 = sorted[Math.floor(sorted.length * 0.5)];
const p95 = sorted[Math.floor(sorted.length * 0.95)];
const avg = frameTimes.reduce((a, b) => a + b, 0) / frameTimes.length;
console.log(`FPS P50: ${(1000/p50).toFixed(1)}, P95: ${(1000/p95).toFixed(1)}, Avg: ${(1000/avg).toFixed(1)}`);
