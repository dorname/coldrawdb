import { appendFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const currentDir = dirname(fileURLToPath(import.meta.url));
const resultPath = resolve(currentDir, "../../../../logos/resources/verify/test-results.jsonl");

export function reportOpenLogos({ id, status, durationMs, error }) {
  if (!/^(UT|ST)-[^-]+-.+$/.test(id)) throw new Error(`无效的 OpenLogos 用例 ID：${id}`);
  if (!["pass", "fail", "skip"].includes(status)) throw new Error(`无效的 OpenLogos 状态：${status}`);
  if (status === "fail" && !error) throw new Error("失败记录必须包含 error");

  const record = {
    id,
    status,
    duration_ms: Math.round(durationMs),
    timestamp: new Date().toISOString(),
    ...(error ? { error } : {}),
  };
  mkdirSync(dirname(resultPath), { recursive: true });
  appendFileSync(resultPath, `${JSON.stringify(record)}\n`, "utf8");
}
