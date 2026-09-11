import { mkdirSync, readdirSync, statSync, unlinkSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const currentDir = dirname(fileURLToPath(import.meta.url));

function walkFiles(root, visit) {
  if (!root) return;
  const stack = [root];
  while (stack.length > 0) {
    const dir = stack.pop();
    let entries;
    try {
      entries = readdirSync(dir);
    } catch {
      continue;
    }
    for (const name of entries) {
      const path = join(dir, name);
      let stat;
      try {
        stat = statSync(path);
      } catch {
        continue;
      }
      if (stat.isDirectory()) stack.push(path);
      else visit(name, path);
    }
  }
}

function inspectBrowserDir(dir) {
  let headless = null;
  let chrome = null;
  walkFiles(dir, (name, path) => {
    if (name === "chrome-headless-shell") headless = path;
    if (name === "chrome" && path.includes("chrome-linux")) chrome = path;
  });
  return { dir, headless, chrome };
}

function isWritableDir(dir) {
  try {
    mkdirSync(dir, { recursive: true });
    const probe = join(dir, `.write-probe-${process.pid}`);
    writeFileSync(probe, "ok");
    unlinkSync(probe);
    return true;
  } catch {
    return false;
  }
}

function candidateDirs() {
  const dirs = [];
  const seen = new Set();
  const push = dir => {
    if (!dir || seen.has(dir)) return;
    seen.add(dir);
    dirs.push(dir);
  };
  push(process.env.PLAYWRIGHT_BROWSERS_PATH);
  push(join(homedir(), ".cache/ms-playwright"));
  push(join(tmpdir(), "coldrawdb-ms-playwright"));
  return dirs;
}

function pickBrowser(dirs) {
  const inspected = dirs.map(inspectBrowserDir);
  return (
    inspected.find(item => item.headless) ??
    inspected.find(item => item.chrome) ??
    null
  );
}

function installBrowsers(dest) {
  const result = spawnSync("npx", ["playwright", "install", "chromium"], {
    cwd: currentDir,
    stdio: "inherit",
    env: { ...process.env, PLAYWRIGHT_BROWSERS_PATH: dest },
  });
  if (result.status !== 0) {
    throw new Error(`npx playwright install chromium 失败（exit=${result.status ?? "null"}）`);
  }
}

export function applyPlaywrightBrowserEnv() {
  let chosen = pickBrowser(candidateDirs());
  if (!chosen) {
    const dest = [join(homedir(), ".cache/ms-playwright"), join(tmpdir(), "coldrawdb-ms-playwright")].find(isWritableDir);
    if (!dest) {
      throw new Error("找不到可写的 Playwright 浏览器缓存目录");
    }
    process.stderr.write(`[playwright] 未发现可用浏览器，安装到 ${dest}\n`);
    installBrowsers(dest);
    chosen = pickBrowser([dest]);
  }
  if (!chosen?.headless && !chosen?.chrome) {
    throw new Error("Playwright 浏览器解析失败：未找到 chrome-headless-shell 或 chromium");
  }
  process.env.PLAYWRIGHT_BROWSERS_PATH = chosen.dir;
  if (!chosen.headless && chosen.chrome) {
    process.env.PLAYWRIGHT_CHROMIUM_USE_HEADLESS_SHELL = "0";
  }
  return chosen;
}

if (import.meta.url === `file://${process.argv[1]}` || process.argv.includes("--export-env")) {
  const chosen = applyPlaywrightBrowserEnv();
  if (process.argv.includes("--export-env")) {
    process.stdout.write(`export PLAYWRIGHT_BROWSERS_PATH='${chosen.dir}'\n`);
    if (!chosen.headless && chosen.chrome) {
      process.stdout.write("export PLAYWRIGHT_CHROMIUM_USE_HEADLESS_SHELL=0\n");
    }
  } else {
    process.stdout.write(
      JSON.stringify(
        {
          browsersPath: chosen.dir,
          headless: Boolean(chosen.headless),
          chrome: Boolean(chosen.chrome),
        },
        null,
        2,
      ) + "\n",
    );
  }
}
