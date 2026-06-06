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
  const usedMB = (metrics.usedJSHeapSize / 1048576).toFixed(1);
  const totalMB = (metrics.totalJSHeapSize / 1048576).toFixed(1);
  console.log(`Heap Used: ${usedMB}MB, Total: ${totalMB}MB`);
} else {
  console.log('performance.memory not available (Chromium only with --enable-precise-memory-info)');
}
