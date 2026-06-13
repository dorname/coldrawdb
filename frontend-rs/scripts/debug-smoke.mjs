import { chromium } from "playwright";

const BASE_URL = "http://localhost:8080";

async function waitEditorReady(page) {
  await page.goto(BASE_URL + "/");
  await page.waitForSelector('[data-testid="editor-ready"]', { timeout: 20_000 });
  await page.waitForLoadState("networkidle", { timeout: 5_000 }).catch(() => {});
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await ctx.newPage();

  await waitEditorReady(page);
  await page.locator('[data-testid="top-menu-bar"]').waitFor({ state: "visible" });
  await page.locator('[data-testid="toolbar"]').waitFor({ state: "visible" });
  await page.locator('[data-testid="left-panel"]').waitFor({ state: "visible" });
  await page.locator('[data-testid="editor-canvas"]').waitFor({ state: "visible" });
  await page.locator('[data-testid="revision-display"]').waitFor({ state: "visible" });

  const btn = page.locator('[data-testid="btn-create-table"]');
  console.log("btn count:", await btn.count());
  console.log("btn visible:", await btn.isVisible().catch(() => false));
  console.log("btn bounding box:", await btn.boundingBox());
  console.log("btn text:", await btn.textContent());

  await ctx.close();
  await browser.close();
}

main().catch(console.error);
