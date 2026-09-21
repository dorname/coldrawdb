# 代码就绪稿（merge 后立即落地）

> 提案：fix-table-header-contrast | 规格：R-COLOR-04 / UT-CR-COLOR-03
> 触发条件：`SPEC_MERGED` 存在后执行；本文件非正式规格。

## A. 插入位置：`editor_render.rs`（紧接 `table_border_color` 之后）

```rust
/// R-COLOR-04：表头有效背景相对亮度阈值（WCAG sRGB 相对亮度）。
pub const HEADER_FG_LUMINANCE_THRESHOLD: f64 = 0.55;

/// 表头强/次前景色对（R-COLOR-04）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderForeground<'a> {
    pub strong: &'a str,
    pub muted: &'a str,
}

fn srgb_channel_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance_rgb(r: f64, g: f64, b: f64) -> f64 {
    let r = srgb_channel_to_linear(r);
    let g = srgb_channel_to_linear(g);
    let b = srgb_channel_to_linear(b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn parse_hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn parse_hex_byte(h: u8, l: u8) -> Option<f64> {
    Some(((parse_hex_digit(h)? << 4) | parse_hex_digit(l)?) as f64 / 255.0)
}

/// 解析 `#rgb` / `#rrggbb` / `rgb(r,g,b)` / `rgba(r,g,b,a)` → (r,g,b,a) ∈ 0..=1。
pub fn parse_css_color_rgba(input: &str) -> Option<(f64, f64, f64, f64)> {
    let s = input.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let b = hex.as_bytes();
        return match b.len() {
            3 => Some((
                parse_hex_digit(b[0])? as f64 / 15.0,
                parse_hex_digit(b[1])? as f64 / 15.0,
                parse_hex_digit(b[2])? as f64 / 15.0,
                1.0,
            )),
            6 => Some((
                parse_hex_byte(b[0], b[1])?,
                parse_hex_byte(b[2], b[3])?,
                parse_hex_byte(b[4], b[5])?,
                1.0,
            )),
            8 => Some((
                parse_hex_byte(b[0], b[1])?,
                parse_hex_byte(b[2], b[3])?,
                parse_hex_byte(b[4], b[5])?,
                parse_hex_byte(b[6], b[7])?,
            )),
            _ => None,
        };
    }
    let lower = s.to_ascii_lowercase();
    let (body, has_a) = if let Some(rest) = lower.strip_prefix("rgba(") {
        (rest.strip_suffix(')')?, true)
    } else if let Some(rest) = lower.strip_prefix("rgb(") {
        (rest.strip_suffix(')')?, false)
    } else {
        return None;
    };
    let parts: Vec<&str> = body.split(',').map(str::trim).collect();
    if has_a {
        if parts.len() != 4 {
            return None;
        }
        let r: f64 = parts[0].parse().ok()?;
        let g: f64 = parts[1].parse().ok()?;
        let b: f64 = parts[2].parse().ok()?;
        let a: f64 = parts[3].parse().ok()?;
        Some((r / 255.0, g / 255.0, b / 255.0, a.clamp(0.0, 1.0)))
    } else {
        if parts.len() != 3 {
            return None;
        }
        let r: f64 = parts[0].parse().ok()?;
        let g: f64 = parts[1].parse().ok()?;
        let b: f64 = parts[2].parse().ok()?;
        Some((r / 255.0, g / 255.0, b / 255.0, 1.0))
    }
}

fn composite_over(
    src: (f64, f64, f64, f64),
    dst: (f64, f64, f64, f64),
) -> (f64, f64, f64) {
    let (sr, sg, sb, sa) = src;
    let (dr, dg, db, da) = dst;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 1e-9 {
        return (dr, dg, db);
    }
    let r = (sr * sa + dr * da * (1.0 - sa)) / out_a;
    let g = (sg * sa + dg * da * (1.0 - sa)) / out_a;
    let b = (sb * sa + db * da * (1.0 - sa)) / out_a;
    (r, g, b)
}

/// tint over base 合成后的相对亮度；任一色解析失败 → None。
pub fn composited_relative_luminance(tint: &str, base: &str) -> Option<f64> {
    let src = parse_css_color_rgba(tint)?;
    let dst = parse_css_color_rgba(base).unwrap_or((0.0, 0.0, 0.0, 1.0));
    let (r, g, b) = composite_over(src, dst);
    Some(relative_luminance_rgb(r, g, b))
}

/// R-COLOR-04：按表头有效背景亮度选择深/浅前景对。
/// `fallback_dark_fg=true` 表示解析失败时倾向深色字（亮主题回退）。
pub fn header_foreground_colors<'a>(
    header_tint: &str,
    table_bg: &str,
    light_strong: &'a str,
    light_muted: &'a str,
    dark_strong: &'a str,
    dark_muted: &'a str,
    fallback_dark_fg: bool,
) -> HeaderForeground<'a> {
    let use_dark_fg = composited_relative_luminance(header_tint, table_bg)
        .map(|l| l >= HEADER_FG_LUMINANCE_THRESHOLD)
        .unwrap_or(fallback_dark_fg);
    if use_dark_fg {
        HeaderForeground {
            strong: light_strong,
            muted: light_muted,
        }
    } else {
        HeaderForeground {
            strong: dark_strong,
            muted: dark_muted,
        }
    }
}
```

## B. `draw_table_body` 表头取色改写

在 `let header_tint = ...` 之后、绘制表名之前：

```rust
let fg = header_foreground_colors(
    header_tint,
    palette.table_bg,
    PALETTE_LIGHT.text_strong,
    PALETTE_LIGHT.text_muted,
    PALETTE_DARK.text_strong,
    PALETTE_DARK.text_muted,
    !current_theme_dark(), // 亮主题失败回退深色字
);
```

- 表名：`fg.strong`（替换 `palette.text_strong`）
- 旁注 / 字段计数：`fg.muted`（替换表头两处 `palette.text_muted`）
- 字段行保持 `palette.text_*` 不变

## C. UT — 追加到 `canvas_comment_color_ut.rs`

```rust
use frontend_rs::editor_render::{
    header_foreground_colors, composited_relative_luminance, parse_css_color_rgba,
    HEADER_FG_LUMINANCE_THRESHOLD, /* existing imports */
};

#[test]
fn ut_cr_color_03_header_contrast_adaptive() {
    let start = Instant::now();
    let ls = "#142c34";
    let lm = "#7b8d93";
    let ds = "#f2fdfe";
    let dm = "#86a3ab";
    let dark_bg = "rgba(16,38,45,.94)";
    let light_bg = "rgba(255,255,255,.84)";

    // 1) 浅色实色表头 → 深色字
    let fg = header_foreground_colors("#e8eef0", dark_bg, ls, lm, ds, dm, false);
    assert_eq!(fg.strong, ls);
    assert_eq!(fg.muted, lm);
    let fg = header_foreground_colors("#ffffff", dark_bg, ls, lm, ds, dm, false);
    assert_eq!(fg.strong, ls);

    // 2) 深色实色表头 → 浅色字
    let fg = header_foreground_colors("#175e7a", light_bg, ls, lm, ds, dm, true);
    assert_eq!(fg.strong, ds);
    let fg = header_foreground_colors("#142c34", light_bg, ls, lm, ds, dm, true);
    assert_eq!(fg.strong, ds);

    // 3) 暗主题默认半透明 tint over 暗底 → 浅色字
    let l = composited_relative_luminance("rgba(79,209,197,.18)", dark_bg).unwrap();
    assert!(l < HEADER_FG_LUMINANCE_THRESHOLD);
    let fg = header_foreground_colors("rgba(79,209,197,.18)", dark_bg, ls, lm, ds, dm, false);
    assert_eq!(fg.strong, ds);

    // 4) 亮主题默认 tint over 亮底 → 深色字
    let l = composited_relative_luminance("rgba(30,131,147,.13)", light_bg).unwrap();
    assert!(l >= HEADER_FG_LUMINANCE_THRESHOLD);
    let fg = header_foreground_colors("rgba(30,131,147,.13)", light_bg, ls, lm, ds, dm, true);
    assert_eq!(fg.strong, ls);

    // 5) 表色接近暗主题字色 → 必须深色字
    let fg = header_foreground_colors("#f2fdfe", dark_bg, ls, lm, ds, dm, false);
    assert_eq!(fg.strong, ls, "近白表头不得继续用浅色字");

    // 6) 渲染锚点
    let body = RENDER.find("fn draw_table_body").expect("draw_table_body");
    let block = &RENDER[body..body + 2500.min(RENDER.len() - body)];
    assert!(
        block.contains("header_foreground_colors("),
        "R-COLOR-04：表头文字必须经 header_foreground_colors"
    );

    let _ = parse_css_color_rgba("#abc");
    report("UT-CR-COLOR-03", start);
}
```

## D. `openlogos_reporter.rs`

在 `"UT-CR-COLOR-02"` 旁追加 `"UT-CR-COLOR-03"`。

## E. 验证

```bash
cd /home/kyle/coldrawdb/frontend-rs && cargo test --test canvas_comment_color_ut ut_cr_color_03 -- --nocapture
```

---

**当前阻塞**：需用户明确授权 `openlogos merge fix-table-header-contrast` 后才能合并规格并应用本稿。
