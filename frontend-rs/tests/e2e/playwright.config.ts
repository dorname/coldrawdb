import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright config for coldrawdb V2 main paths.
 *
 * change-20260826-1330-complete-skipped-e2e:
 * - 覆盖 ST-FE-S03-01~05（鉴权）/ S04-01~06（房间）/ S05-01~06（OT 协作）
 * - 覆盖 ST-FE-V2-01~04（V2 全链路）
 * - 启动 trunk serve + cargo run backend 在 webServer 阶段
 *
 * 浏览器：仅 Chromium（headless shell）+ --font-render-hinting=none
 *        确保字体清晰度回归可重现（DPR 缩放）。
 *
 * Reporter：默认 spec + list；OpenLogos reporter 由 reporter/openlogos.ts 注入
 *        （见 reporter/openlogos.ts）。
 */
export default defineConfig({
  testDir: "./specs",
  fullyParallel: false, // 同一后端的 e2e 必须串行避免端口冲突
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: [
    ["list"],
    ["./reporter/openlogos.ts", { outputFile: "../../logos/resources/verify/test-results.jsonl" }],
  ],
  use: {
    baseURL: process.env.E2E_BASE_URL ?? "http://127.0.0.1:18080",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    launchOptions: {
      args: [
        "--font-render-hinting=none",
        "--force-device-scale-factor=1", // 强制 dpr=1，避免截图因 HiDPI 漂移
      ],
    },
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: {
    command: "cd ../.. && bash scripts/start-local.sh",
    url: "http://127.0.0.1:18080/editor",
    timeout: 120_000,
    reuseExistingServer: !process.env.CI,
    stdout: "pipe",
    stderr: "pipe",
  },
});