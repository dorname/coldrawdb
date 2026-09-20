//! 力导向布局（Fruchterman-Reingold 变体）— 对齐 MCP `force_directed_layout`。
//! 纯函数、确定性输出（固定随机种子 42）、无副作用。
//! Spec: core-01b-relationship.md §4.5 / layout-after-import-command / #23

use crate::editor_core::types::{Reference, Table};

/// 力导向布局参数（默认与 MCP LayoutParams::default 对齐）。
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

/// 执行力导向布局，返回更新后的 tables。
///
/// - 仅修改连通表的 `x`/`y`；其他字段不变
/// - 孤立表（无关联边）位置不变
/// - 无边时原样返回
/// - 固定随机种子 42，相同输入必然相同输出
pub fn force_directed_layout(
    tables: &[Table],
    references: &[Reference],
    params: &LayoutParams,
) -> Vec<Table> {
    let n = tables.len();
    if n == 0 {
        return tables.to_vec();
    }

    let mut positions: Vec<(f64, f64)> = tables.iter().map(|t| (t.x, t.y)).collect();

    let edges: Vec<(usize, usize)> = references
        .iter()
        .filter_map(|r| {
            let start_idx = tables.iter().position(|t| t.id == r.start_table_id)?;
            let end_idx = tables.iter().position(|t| t.id == r.end_table_id)?;
            Some((start_idx, end_idx))
        })
        .collect();

    if edges.is_empty() {
        return tables.to_vec();
    }

    let area = params.spacing * params.spacing * n as f64;
    let k = (area / n as f64).sqrt();
    let mut temperature = params.spacing * 0.3;
    let cooling = temperature / params.iterations as f64;

    let mut rng_state: u64 = 42;

    for _iter in 0..params.iterations {
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

        for i in 0..n {
            let is_isolated = !edges.iter().any(|&(a, b)| a == i || b == i);
            if is_isolated {
                continue;
            }
            let disp_mag = (disp[i].0 * disp[i].0 + disp[i].1 * disp[i].1).sqrt();
            if disp_mag > 0.01 {
                let scale = temperature.min(disp_mag) / disp_mag;
                positions[i].0 += disp[i].0 * scale;
                positions[i].1 += disp[i].1 * scale;
            }
        }

        temperature = (temperature - cooling).max(0.01);

        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let jitter = ((rng_state >> 33) as f64 / u64::MAX as f64 - 0.5) * 0.1;
        for pos in positions.iter_mut() {
            pos.0 += jitter;
            pos.1 += jitter;
        }
    }

    tables
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let mut updated = t.clone();
            updated.x = positions[i].0;
            updated.y = positions[i].1;
            updated
        })
        .collect()
}

/// 默认参数下的便捷入口（iterations=100, spacing=180）。
pub fn apply_default_layout(tables: &[Table], references: &[Reference]) -> Vec<Table> {
    force_directed_layout(tables, references, &LayoutParams::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_core::types::Field;

    fn make_table(id: &str, x: f64, y: f64) -> Table {
        Table {
            id: id.into(),
            name: id.into(),
            x,
            y,
            color: String::new(),
            comment: String::new(),
            fields: vec![Field {
                id: format!("{id}_f"),
                name: "id".into(),
                type_: "INT".into(),
                default: String::new(),
                check: String::new(),
                primary: true,
                unique: false,
                not_null: true,
                increment: false,
                comment: String::new(),
                tag: String::new(),
                dict_code: String::new(),
            }],
            indices: vec![],
            width: None,
            min_height: None,
        }
    }

    fn make_ref(start: &str, end: &str) -> Reference {
        Reference {
            id: format!("{start}_{end}"),
            name: format!("{start}->{end}"),
            start_table_id: start.into(),
            end_table_id: end.into(),
            start_field_id: format!("{start}_f"),
            end_field_id: format!("{end}_f"),
            type_: "one_to_many".into(),
            on_delete: "RESTRICT".into(),
            on_update: "RESTRICT".into(),
            color: String::new(),
            line_type: String::new(),
            stroke_style: String::new(),
        }
    }

    /// UT-PB-16 / UT-CR-LAYOUT-01：确定性
    #[test]
    fn ut_pb_16_deterministic() {
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

    /// UT-PB-16 / UT-CR-LAYOUT-01：无重叠
    #[test]
    fn ut_pb_16_no_overlap() {
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
        for i in 0..result.len() {
            for j in (i + 1)..result.len() {
                let dx = result[i].x - result[j].x;
                let dy = result[i].y - result[j].y;
                let dist = (dx * dx + dy * dy).sqrt();
                assert!(
                    dist > 50.0,
                    "表 {} 和 {} 距离过近: {}",
                    result[i].id,
                    result[j].id,
                    dist
                );
            }
        }
    }

    /// UT-PB-16 / UT-CR-LAYOUT-01：孤立表不动
    #[test]
    fn ut_pb_16_isolated_unchanged() {
        let tables = vec![
            make_table("a", 0.0, 0.0),
            make_table("b", 100.0, 0.0),
            make_table("isolated", 500.0, 500.0),
        ];
        let refs = vec![make_ref("a", "b")];
        let result = force_directed_layout(&tables, &refs, &LayoutParams::default());
        let iso = result.iter().find(|t| t.id == "isolated").unwrap();
        assert!((iso.x - 500.0).abs() < 5.0);
        assert!((iso.y - 500.0).abs() < 5.0);
    }

    #[test]
    fn ut_pb_16_empty_edges_unchanged() {
        let tables = vec![make_table("a", 10.0, 20.0), make_table("b", 30.0, 40.0)];
        let result = force_directed_layout(&tables, &[], &LayoutParams::default());
        assert_eq!(result, tables);
    }

    #[test]
    fn ut_pb_16_empty_tables() {
        let result = force_directed_layout(&[], &[], &LayoutParams::default());
        assert!(result.is_empty());
    }
}
