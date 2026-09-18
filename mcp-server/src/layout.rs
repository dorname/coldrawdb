//! mcp-canvas-tools（2026-09-18）：力导向布局算法（Fruchterman-Reingold 变体）。
//! 纯函数、确定性输出（固定随机种子）、无副作用。

use serde_json::{json, Value};

/// 力导向布局参数
#[derive(Debug, Clone)]
pub struct LayoutParams {
    pub iterations: usize,
    pub spacing: f64,
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            iterations: 100,
            spacing: 180.0,
        }
    }
}

/// 执行力导向布局，返回更新后的 tables 数组。
///
/// # 参数
/// - `tables`: 表数组，每个元素需含 `id`、`x`、`y` 字段
/// - `references`: 关系边数组，每个元素需含 `start_table_id`、`end_table_id`
/// - `params`: 布局参数（迭代次数、间距）
///
/// # 返回值
/// 更新后的 tables 数组（仅修改 `x`/`y`，其他属性保持不变）。
/// 孤立表（无关联边）位置不变。
///
/// # 确定性保证
/// 使用固定随机种子（42），相同输入必然产生相同输出。
pub fn force_directed_layout(
    tables: &[Value],
    references: &[Value],
    params: &LayoutParams,
) -> Vec<Value> {
    let n = tables.len();
    if n == 0 {
        return tables.to_vec();
    }

    // 提取初始位置
    let mut positions: Vec<(f64, f64)> = tables
        .iter()
        .map(|t| {
            let x = t.get("x").and_then(Value::as_f64).unwrap_or(0.0);
            let y = t.get("y").and_then(Value::as_f64).unwrap_or(0.0);
            (x, y)
        })
        .collect();

    // 构建关联边索引对
    let edges: Vec<(usize, usize)> = references
        .iter()
        .filter_map(|r| {
            let start_id = r.get("start_table_id").and_then(Value::as_str)?;
            let end_id = r.get("end_table_id").and_then(Value::as_str)?;
            let start_idx = tables.iter().position(|t| {
                t.get("id").and_then(Value::as_str) == Some(start_id)
            })?;
            let end_idx = tables.iter().position(|t| {
                t.get("id").and_then(Value::as_str) == Some(end_id)
            })?;
            Some((start_idx, end_idx))
        })
        .collect();

    // 无边时直接返回原位置
    if edges.is_empty() {
        return tables.to_vec();
    }

    // Fruchterman-Reingold 参数
    let area = params.spacing * params.spacing * n as f64;
    let k = (area / n as f64).sqrt(); // 理想边长
    let mut temperature = params.spacing * 0.3; // 初始温度
    let cooling = temperature / params.iterations as f64;

    // 固定随机种子（确定性）
    let mut rng_state: u64 = 42;

    for _iter in 0..params.iterations {
        // 斥力（所有节点对）
        let mut disp: Vec<(f64, f64)> = vec![(0.0, 0.0); n];
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = positions[i].0 - positions[j].0;
                let dy = positions[i].1 - positions[j].1;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = k * k / dist;
                let fx = dx / dist * force;
                let fy = dy / dist * force;
                disp[i].0 += fx;
                disp[i].1 += fy;
                disp[j].0 -= fx;
                disp[j].1 -= fy;
            }
        }

        // 引力（边）
        for &(i, j) in &edges {
            let dx = positions[i].0 - positions[j].0;
            let dy = positions[i].1 - positions[j].1;
            let dist = (dx * dx + dy * dy).sqrt().max(0.01);
            let force = dist * dist / k;
            let fx = dx / dist * force;
            let fy = dy / dist * force;
            disp[i].0 -= fx;
            disp[i].1 -= fy;
            disp[j].0 += fx;
            disp[j].1 += fy;
        }

        // 更新位置（带温度限制）；孤立表（无边连接）位置不变
        for i in 0..n {
            let is_isolated = !edges.iter().any(|&(a, b)| a == i || b == i);
            if is_isolated {
                continue; // 孤立表不移动
            }
            let disp_mag = (disp[i].0 * disp[i].0 + disp[i].1 * disp[i].1).sqrt();
            if disp_mag > 0.01 {
                let scale = temperature.min(disp_mag) / disp_mag;
                positions[i].0 += disp[i].0 * scale;
                positions[i].1 += disp[i].1 * scale;
            }
        }

        // 冷却
        temperature = (temperature - cooling).max(0.01);

        // 微小随机扰动（打破对称，固定种子保证确定性）
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let jitter = ((rng_state >> 33) as f64 / u64::MAX as f64 - 0.5) * 0.1;
        for pos in positions.iter_mut() {
            pos.0 += jitter;
            pos.1 += jitter;
        }
    }

    // 构建输出：仅更新 x/y，保留其他属性
    tables
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut updated = t.clone();
            updated["x"] = json!(positions[i].0);
            updated["y"] = json!(positions[i].1);
            updated
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_table(id: &str, x: f64, y: f64) -> Value {
        json!({"id": id, "x": x, "y": y, "name": id})
    }

    fn make_ref(start: &str, end: &str) -> Value {
        json!({"start_table_id": start, "end_table_id": end})
    }

    #[test]
    fn test_deterministic_output() {
        let tables = vec![
            make_table("a", 0.0, 0.0),
            make_table("b", 100.0, 0.0),
            make_table("c", 50.0, 100.0),
        ];
        let refs = vec![make_ref("a", "b"), make_ref("b", "c")];
        let params = LayoutParams::default();

        let r1 = force_directed_layout(&tables, &refs, &params);
        let r2 = force_directed_layout(&tables, &refs, &params);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_no_overlap() {
        let tables = vec![
            make_table("a", 0.0, 0.0),
            make_table("b", 10.0, 10.0),
            make_table("c", 20.0, 20.0),
        ];
        let refs = vec![make_ref("a", "b"), make_ref("b", "c")];
        let params = LayoutParams {
            iterations: 100,
            spacing: 180.0,
        };

        let result = force_directed_layout(&tables, &refs, &params);
        // 验证最小间距（不应有表重叠）
        for i in 0..result.len() {
            for j in (i + 1)..result.len() {
                let dx = result[i]["x"].as_f64().unwrap() - result[j]["x"].as_f64().unwrap();
                let dy = result[i]["y"].as_f64().unwrap() - result[j]["y"].as_f64().unwrap();
                let dist = (dx * dx + dy * dy).sqrt();
                assert!(dist > 50.0, "表 {} 和 {} 距离过近: {}", i, j, dist);
            }
        }
    }

    #[test]
    fn test_isolated_table_unchanged() {
        let tables = vec![
            make_table("a", 0.0, 0.0),
            make_table("b", 100.0, 0.0),
            make_table("isolated", 500.0, 500.0),
        ];
        let refs = vec![make_ref("a", "b")]; // isolated 无边
        let params = LayoutParams::default();

        let result = force_directed_layout(&tables, &refs, &params);
        let iso = result.iter().find(|t| t["id"] == "isolated").unwrap();
        // 孤立表位置应保持不变（允许微小扰动）
        let x = iso["x"].as_f64().unwrap();
        let y = iso["y"].as_f64().unwrap();
        assert!((x - 500.0).abs() < 5.0);
        assert!((y - 500.0).abs() < 5.0);
    }

    #[test]
    fn test_empty_tables() {
        let result = force_directed_layout(&[], &[], &LayoutParams::default());
        assert!(result.is_empty());
    }

    #[test]
    fn test_empty_edges() {
        let tables = vec![make_table("a", 10.0, 20.0), make_table("b", 30.0, 40.0)];
        let result = force_directed_layout(&tables, &[], &LayoutParams::default());
        assert_eq!(result, tables);
    }
}
