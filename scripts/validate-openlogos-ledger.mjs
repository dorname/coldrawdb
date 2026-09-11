import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { reportOpenLogos } from "../frontend-rs/tests/e2e/helpers/openlogos-reporter.mjs";

const startedAt = Date.now();
const currentDir = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(currentDir, "..");
const testRoot = join(projectRoot, "logos/resources/test");
const resultPath = join(projectRoot, "logos/resources/verify/test-results.jsonl");
const reportIndex = process.argv.indexOf("--report");
const selfReportId = reportIndex >= 0 ? process.argv[reportIndex + 1] : null;

function markdownFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return markdownFiles(path);
    return entry.isFile() && entry.name.endsWith(".md") ? [path] : [];
  });
}

function readDefinedCases() {
  const cases = new Map();
  const rowPattern = /^\|\s*((?:UT|ST)-[A-Z0-9-]+)(\s+\[manual\])?\s*\|/;
  for (const path of markdownFiles(testRoot)) {
    for (const line of readFileSync(path, "utf8").split(/\r?\n/)) {
      const match = line.match(rowPattern);
      if (!match) continue;
      cases.set(match[1], { manual: Boolean(match[2]), path });
    }
  }
  return cases;
}

function readLatestResults() {
  const results = new Map();
  const lines = readFileSync(resultPath, "utf8").split(/\r?\n/).filter(Boolean);
  for (const [index, line] of lines.entries()) {
    let record;
    try {
      record = JSON.parse(line);
    } catch (error) {
      throw new Error(`测试账本第 ${index + 1} 行不是合法 JSON：${error.message}`);
    }
    assert.match(record.id, /^(UT|ST)-[A-Z0-9-]+$/, `第 ${index + 1} 行用例 ID 无效`);
    assert.ok(["pass", "fail", "skip"].includes(record.status), `第 ${index + 1} 行状态无效`);
    results.set(record.id, record);
  }
  return results;
}

function validateLedger(defined, results, ignoredIds = new Set()) {
  const unknown = [...results.keys()].filter(id => !defined.has(id));
  const uncovered = [...defined.entries()]
    .filter(([id, item]) => !item.manual && !ignoredIds.has(id) && !results.has(id))
    .map(([id]) => id);
  const failed = [...results.values()].filter(record => record.status === "fail").map(record => record.id);
  assert.deepEqual(unknown, [], `存在未登记 reporter ID：${unknown.join(", ")}`);
  assert.deepEqual(uncovered, [], `存在未覆盖自动化用例：${uncovered.join(", ")}`);
  assert.deepEqual(failed, [], `存在失败用例：${failed.join(", ")}`);
}

const defined = readDefinedCases();
const results = readLatestResults();
const ignoredIds = new Set(selfReportId ? [selfReportId] : []);
validateLedger(defined, results, ignoredIds);

if (selfReportId) {
  assert.ok(defined.has(selfReportId), `自报告用例未登记：${selfReportId}`);
  reportOpenLogos({
    id: selfReportId,
    status: "pass",
    durationMs: Date.now() - startedAt,
  });
  results.set(selfReportId, { id: selfReportId, status: "pass" });
  validateLedger(defined, results);
}

const manualCount = [...defined.values()].filter(item => item.manual).length;
process.stdout.write(
  `${JSON.stringify({
    defined: defined.size,
    manual: manualCount,
    automated: defined.size - manualCount,
    executed: results.size,
    status: "PASS",
  })}\n`,
);
