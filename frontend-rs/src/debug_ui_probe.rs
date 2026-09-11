//! 运行时 UI 样式探测（Debug session 1b2005）
use gloo_timers::callback::Timeout;
use leptos::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::{window, Document, Element};

const INGEST: &str = "http://127.0.0.1:7633/ingest/e9d01c94-8730-4ccb-b3d4-c4380e977e12";

fn css_prop(el: &Element, prop: &str) -> String {
    window()
        .and_then(|w| w.get_computed_style(el).ok())
        .flatten()
        .and_then(|s| s.get_property_value(prop).ok())
        .unwrap_or_else(|| "n/a".into())
}

fn stylesheet_hrefs(doc: &Document) -> Vec<String> {
    let mut hrefs = Vec::new();
    if let Ok(list) = doc.query_selector_all("link[rel=\"stylesheet\"]") {
        for i in 0..list.length() {
            if let Some(node) = list.item(i) {
                if let Some(link) = node.dyn_ref::<web_sys::HtmlLinkElement>() {
                    hrefs.push(link.href());
                }
            }
        }
    }
    hrefs
}

// #region agent log
fn post_log(hypothesis_id: &str, location: &str, message: &str, data: &str) {
    post_log_with_run("post-fix", hypothesis_id, location, message, data);
}

fn post_log_with_run(run_id: &str, hypothesis_id: &str, location: &str, message: &str, data: &str) {
    let body = format!(
        r#"{{"sessionId":"1b2005","runId":"{run_id}","hypothesisId":"{hypothesis_id}","location":"{location}","message":"{message}","data":{data},"timestamp":{ts}}}"#,
        run_id = run_id,
        hypothesis_id = hypothesis_id,
        location = location,
        message = message.replace('"', "\\\""),
        data = data,
        ts = js_sys::Date::now() as u64
    );
    spawn_local(async move {
        if let Ok(req) = gloo_net::http::Request::post(INGEST)
            .header("Content-Type", "application/json")
            .header("X-Debug-Session-Id", "1b2005")
            .body(body)
        {
            let _ = req.send().await;
        }
    });
}
// #endregion

fn probe_once() {
    let win = match window() {
        Some(w) => w,
        None => return,
    };
    let doc = match win.document() {
        Some(d) => d,
        None => return,
    };

    let html_el = doc.document_element();
    let data_mode = html_el
        .as_ref()
        .and_then(|el| el.get_attribute("data-mode"))
        .unwrap_or_else(|| "(unset)".into());
    let prefers_dark = win
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false);

    let sheet_hrefs = stylesheet_hrefs(&doc);
    let probe_el: Element = html_el
        .map(|e| e.into())
        .or_else(|| doc.body().map(|b| b.into()))
        .unwrap();
    let root_glass = css_prop(&probe_el, "--cdb-glass-bg");

    let empty_info = if let Ok(Some(el)) = doc.query_selector("[data-testid=\"canvas-empty-guide\"]") {
        format!(
            r#"{{"exists":true,"position":"{}","top":"{}","left":"{}","transform":"{}","background":"{}","backdropFilter":"{}","zIndex":"{}","borderRadius":"{}"}}"#,
            css_prop(&el, "position"),
            css_prop(&el, "top"),
            css_prop(&el, "left"),
            css_prop(&el, "transform"),
            css_prop(&el, "background-color"),
            css_prop(&el, "backdrop-filter"),
            css_prop(&el, "z-index"),
            css_prop(&el, "border-radius"),
        )
    } else {
        r#"{"exists":false}"#.into()
    };

    let app_bar_info = if let Ok(Some(el)) = doc.query_selector("[data-testid=\"app-bar\"]") {
        format!(
            r#"{{"background":"{}","backdropFilter":"{}","boxShadow":"{}"}}"#,
            css_prop(&el, "background-color"),
            css_prop(&el, "backdrop-filter"),
            css_prop(&el, "box-shadow"),
        )
    } else {
        r#"{"exists":false}"#.into()
    };

    let container_info =
        if let Ok(Some(el)) = doc.query_selector("[data-testid=\"editor-canvas-container\"]") {
            let bg_img = css_prop(&el, "background-image");
            let bg_img_short: String = bg_img.chars().take(120).collect();
            format!(
                r#"{{"backgroundColor":"{}","backgroundImagePrefix":"{}","position":"{}"}}"#,
                css_prop(&el, "background-color"),
                bg_img_short.replace('"', "'"),
                css_prop(&el, "position"),
            )
        } else {
            r#"{"exists":false}"#.into()
        };

    let canvas_info = if let Ok(Some(el)) = doc.query_selector("[data-testid=\"editor-canvas\"]") {
        format!(
            r#"{{"background":"{}","zIndex":"{}","flex":"{}"}}"#,
            css_prop(&el, "background-color"),
            css_prop(&el, "z-index"),
            css_prop(&el, "flex"),
        )
    } else {
        r#"{"exists":false}"#.into()
    };

    let stylesheets_json = serde_json::to_string(&sheet_hrefs).unwrap_or_else(|_| "[]".into());

    post_log(
        "A",
        "debug_ui_probe.rs:stylesheets",
        "Stylesheet inventory",
        &format!(
            r#"{{"count":{},"hrefs":{},"hasStylesCss":{}}}"#,
            sheet_hrefs.len(),
            stylesheets_json,
            sheet_hrefs.iter().any(|h| h.contains("styles"))
        ),
    );
    post_log(
        "B",
        "debug_ui_probe.rs:theme",
        "Theme mode probe",
        &format!(
            r#"{{"dataMode":"{}","prefersDark":{},"rootGlassBg":"{}"}}"#,
            data_mode, prefers_dark, root_glass
        ),
    );
    post_log("C", "debug_ui_probe.rs:emptyGuide", "EmptyGuide computed styles", &empty_info);
    post_log("D", "debug_ui_probe.rs:appBar", "AppBar computed styles", &app_bar_info);
    post_log(
        "E",
        "debug_ui_probe.rs:canvasContainer",
        "Canvas container background",
        &container_info,
    );
    post_log("F", "debug_ui_probe.rs:canvasElement", "Canvas element layer", &canvas_info);
}

pub fn schedule_ui_style_probe() {
    Timeout::new(1200, probe_once).forget();
}
