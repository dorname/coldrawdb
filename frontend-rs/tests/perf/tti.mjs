// TTI measurement - Time To Interactive
// Usage: node tests/perf/tti.mjs

import { chromium } from 'playwright';

const browser = await chromium.launch();
const page = await browser.newPage();
const times = [];

for (let i = 0; i < 10; i++) {
  const p = await browser.newPage();
  const start = Date.now();
  await p.goto('http://localhost:8080', { waitUntil: 'domcontentloaded' });
  try {
    await p.waitForSelector('[data-testid=editor-ready]', { timeout: 30000 });
    times.push(Date.now() - start);
  } catch (e) {
    times.push(Date.now() - start);
  }
  await p.close();
}

await browser.close();
const sorted = times.sort((a, b) => a - b);
const p50 = sorted[Math.floor(sorted.length * 0.5)];
const p95 = sorted[Math.floor(sorted.length * 0.95)];
console.log(`TTI P50: ${p50}ms, P95: ${p95}ms, all: ${sorted.join(',')}ms`);