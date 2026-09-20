/**
 * OpenLogos reporter — 把 Playwright 结果转 logos/resources/verify/test-results.jsonl
 *
 * change-20260826-1330-complete-skipped-e2e：每个 spec 文件对应一个 TC ID
 * 前缀（s03-auth.spec.ts → ST-FE-S03-*），test title 对应后缀（01~05）。
 *
 * 字段映射：
 *   playwright passed   → { status: "pass", duration_ms: ms }
 *   playwright failed   → { status: "fail", error: message }
 *   playwright skipped  → { status: "skip", error: reason }
 *   playwright timedout → { status: "fail", error: "timeout" }
 */

import type {
  FullConfig,
  FullResult,
  Reporter,
  TestCase,
  TestResult,
} from "@playwright/test/reporter";
import * as fs from "node:fs";
import * as path from "node:path";

// verify 预跑沙箱（bwrap --ro-bind workspace）下相对路径会指向沙箱副本，
// 预跑脚本已 export COLDRAWDB_JSONL_PATH 为账本绝对路径，优先走该变量。
const OUTPUT =
  process.env.COLDRAWDB_JSONL_PATH ??
  path.resolve(
    import.meta.dirname,
    "../../../../logos/resources/verify/test-results.jsonl",
  );

const PREFIX_MAP: Record<string, string> = {
  "s03-auth.spec.ts": "ST-FE-S03-",
  "s04-rooms.spec.ts": "ST-FE-S04-",
  "s05-collab.spec.ts": "ST-FE-S05-",
  "v2-regression.spec.ts": "ST-FE-V2-",
  // fix-remote-github-issues-7-18（issue #7/#8）：ST-PC-09 拖入 .ddl → 摘要 → 导入画布
  "pc-ddl-drop.spec.ts": "ST-PC-",
  // fix-remote-github-issues-7-18（issue #9/#11）：ST-PB-05 连线换侧 / ST-RP-04 分隔条清晰
  "canvas-sides-resize.spec.ts": "ST-",
  // fix-remote-github-issues-7-18（issue #13/#14/#15）：ST-PE-07 长房间名 AppBar 布局
  "appbar-truncate.spec.ts": "ST-",
  // fix-remote-github-issues-7-18（issue #10/#12）：ST-CR-COMMENT-01 / ST-CR-COLOR-01 / ST-PB-06
  "cr-comment-color.spec.ts": "ST-",
};

function testId(test: TestCase): string | null {
  const file = test.location.file.split("/").pop() ?? "";
  const prefix = PREFIX_MAP[file];
  if (!prefix) return null;

  // test.title 形如 "ST-FE-S03-01: register → 跳转 home" 或 "ST-PC-09: 拖入 .ddl → ..."
  // 多段字母前缀（ST-CR-COMMENT-01 / ST-CR-COLOR-01）：字母段可重复，数字段收尾
  const m = test.title.match(/^(ST-(?:FE-)?[A-Z0-9]+(?:-[A-Z0-9]+)*-\d+)/);
  return m ? m[1] : null;
}

function mapStatus(result: TestResult): { status: string; error?: string } {
  switch (result.status) {
    case "passed":
      return { status: "pass" };
    case "failed":
      return { status: "fail", error: (result.error?.message ?? "").slice(0, 300) };
    case "timedOut":
      return { status: "fail", error: "timeout" };
    case "skipped":
      return { status: "skip", error: "spec-skipped" };
    case "interrupted":
      return { status: "fail", error: "interrupted" };
    default:
      return { status: "skip", error: `unknown-${result.status}` };
  }
}

function isoNow(): string {
  return new Date().toISOString().replace(/\.\d+Z$/, "Z");
}

class OpenLogosReporter implements Reporter {
  async onTestEnd(test: TestCase, result: TestResult) {
    const tcId = testId(test);
    if (!tcId) return; // 跳过不在 PREFIX_MAP 的 spec

    const { status, error } = mapStatus(result);
    const record: Record<string, unknown> = {
      id: tcId,
      status,
      timestamp: isoNow(),
      duration_ms: result.duration,
    };
    if (error) record.error = error;

    fs.mkdirSync(path.dirname(OUTPUT), { recursive: true });
    fs.appendFileSync(OUTPUT, JSON.stringify(record) + "\n");
  }

  async onEnd(_result: FullResult, _config: FullConfig) {
    // No-op；jsonl 由 onTestEnd 流式追加
  }
}

export default OpenLogosReporter;