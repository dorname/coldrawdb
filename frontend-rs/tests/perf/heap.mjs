// Heap memory measurement
// Usage: node tests/perf/heap.mjs

import { chromium } from 'playwright';

const browser = await chromium.launch();
const page = await browser.newPage();
await page.goto('http://localhost:8080', { waitUntil: 'domcontentloaded' });
await page.waitForSelector('[data-testid=editor-ready]', { timeout: 30000 });

// Force GC if available
if (global.gc) global.gc();

const metrics = await page.evaluate(() => {
  if (performance.memory) {
    return {
      usedJSHeapSize: performance.memory.usedJSHeapSize,
      totalJSHeapSize: performance.memory.totalJSHeapSize,
      jsHeapSizeLimit: performance.memory.jsHeapSizeLimit
    };
  }
  return null;
});

await browser.close();
if (metrics) {
  const usedMB = +(metrics.usedJSHeapSize / 1048576).toFixed(2);
  const totalMB = +(metrics.totalJSHeapSize / 1048576).toFixed(2);
  console.log(JSON.stringify({ status: 'ok', usedMB, totalMB }));
} else {
  console.log(JSON.stringify({ status: 'degraded', reason: 'performance.memory not available' }));
}
