#![allow(dead_code)]
#![allow(unused_variables)]

pub mod code_view;
pub mod command_palette;
pub mod components;
pub mod editor_core;
pub mod editor_data_access;
pub mod editor_panels;
pub mod editor_render;
pub mod icons;

use editor_core::{DebounceTrigger, EditorStore};
use editor_data_access::ApiError;
use editor_panels::AppRoot;
use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn mount() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        let store = EditorStore::new();
        let debouncer = DebounceTrigger::default();

        let (diagram_id, share_mode, invite_token) = parse_route_from_location();

        #[cfg(debug_assertions)]
        expose_test_hooks(&store);

        view! {
            <AppRoot store=store debouncer=debouncer _diagram_id=diagram_id share_mode=share_mode invite_token=invite_token />
        }
    });
}

/// 从 URL query `?share=<id>` 解析 diagram id（对齐 S02 Phase 2 / `build_share_url`）。
pub fn parse_share_param(search: &str) -> Option<String> {
    let q = search.strip_prefix('?').unwrap_or(search);
    if q.is_empty() {
        return None;
    }
    for pair in q.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == "share" && !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// 从 pathname 末段解析 diagram id（如 `/editor/d-abc-123`）。
pub fn parse_diagram_id_from_pathname_str(pathname: &str) -> Option<String> {
    pathname
        .split('/')
        .filter(|s| !s.is_empty())
        .next_back()
        .map(|s| s.to_string())
        .filter(|s| s != "editor")
}

/// 纯函数：share 参数优先，其次 pathname，最后 fallback `default`。
pub fn diagram_id_from_location(pathname: &str, search: &str) -> String {
    parse_share_param(search)
        .or_else(|| parse_diagram_id_from_pathname_str(pathname))
        .unwrap_or_else(|| "default".to_string())
}

pub fn route_from_location(pathname: &str, search: &str) -> (String, bool) {
    if let Some(id) = parse_share_param(search) {
        return (id, true);
    }
    (
        parse_diagram_id_from_pathname_str(pathname).unwrap_or_else(|| "default".to_string()),
        false,
    )
}

pub fn route_context_from_location(pathname: &str, search: &str) -> (String, bool, Option<String>) {
    if let Some(token) = parse_invite_token(pathname) {
        return ("default".to_string(), false, Some(token));
    }
    let (id, share_mode) = route_from_location(pathname, search);
    (id, share_mode, None)
}

pub fn parse_invite_token(pathname: &str) -> Option<String> {
    let parts: Vec<&str> = pathname.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() == 2 && parts[0] == "invite" && !parts[1].is_empty() {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// 页面状态机枚举（align-frontend-to-prototype FEUX-AC-01～04）。
///
/// 五种状态，按生产前端与统一主原型 `core-01-editor-prototype.html` 对齐：
/// - `Auth` 未登录默认入口，展示 auth-gate / login-form / auth-tab-register。
/// - `ShareEdit` 匿名只读分享（`?share=<id>`），保持 S02 匿名只读链路。
/// - `Invite` `/invite/{token}` 独立接受页，登录前后均可访问。
/// - `Rooms` 已登录但未进入房间：rooms-list-page，可创建/选择房间。
/// - `RoomEditor` 房间内协作编辑器：`room-editor-page` + tool-rail + canvas + inspector + ot-rev/ws-status/room-presence/activity-feed。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageState {
    Auth,
    ShareEdit,
    Invite,
    Rooms,
    RoomEditor,
}

/// 纯函数：根据 pathname/search 决定初始页面状态。
///
/// 优先级：invite token > share param > 默认 `Auth`（生产前端默认未登录进入 auth）。
pub fn initial_page_state(pathname: &str, search: &str) -> PageState {
    if parse_invite_token(pathname).is_some() {
        return PageState::Invite;
    }
    if parse_share_param(search).is_some() {
        return PageState::ShareEdit;
    }
    PageState::Auth
}

/// 纯函数：脱敏 session notice，禁止写入 token/refresh token/cookie 原文。
///
/// 只允许输出状态文案（"匿名只读分享"、"会话有效"、"登录已过期" 等），不能透出 token。
pub fn sanitize_session_notice(input: Option<&str>) -> Option<String> {
    let s = input?.trim();
    if s.is_empty() {
        return None;
    }
    // 防御：任何看起来像 JWT / 长 base64 / "token=" 子串的提示都丢掉。
    let lower = s.to_lowercase();
    if lower.contains("token=")
        || lower.contains("bearer ")
        || lower.contains("eyj")
        || lower.contains("refresh_token")
        || lower.contains("access_token")
    {
        return None;
    }
    Some(s.to_string())
}

/// 将分享加载失败转换成固定的公开文案，避免透出数据库或私有房间信息。
pub fn share_load_error_message(error: &ApiError) -> String {
    match error {
        ApiError::Server(404, _) => "分享链接不存在或已失效".to_string(),
        ApiError::Network(_) => "暂时无法加载分享图表，请检查网络后重试".to_string(),
        _ => "暂时无法加载分享图表，请稍后重试".to_string(),
    }
}

fn parse_route_from_location() -> (String, bool, Option<String>) {
    web_sys::window()
        .map(|w| {
            route_context_from_location(
                &w.location().pathname().unwrap_or_default(),
                &w.location().search().unwrap_or_default(),
            )
        })
        .unwrap_or_else(|| ("default".to_string(), false, None))
}

#[cfg(debug_assertions)]
fn expose_test_hooks(store: &EditorStore) {
    let root = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element());
    if let Some(root) = root {
        let rev_signal = store.revision;
        let dirty_signal = store.dirty;
        let root_for_rev = root.clone();
        let root_for_dirty = root;
        create_render_effect(move |_| {
            let _ = root_for_rev.set_attribute("data-cdb-revision", &rev_signal.get().to_string());
        });
        create_render_effect(move |_| {
            let _ = root_for_dirty.set_attribute("data-cdb-dirty", &dirty_signal.get().to_string());
        });
    }
}

#[cfg(test)]
mod location_tests {
    use super::{
        diagram_id_from_location, initial_page_state, parse_invite_token, parse_share_param,
        route_context_from_location, route_from_location, sanitize_session_notice,
        share_load_error_message, PageState,
    };
    use crate::editor_data_access::ApiError;

    #[test]
    fn ut_s02_01_share_param_parsed() {
        assert_eq!(
            parse_share_param("?share=abc-123-def"),
            Some("abc-123-def".into())
        );
        assert_eq!(parse_share_param("/editor?share=d-uuid"), None::<String>);
    }

    #[test]
    fn ut_s02_02_share_takes_priority_over_pathname() {
        assert_eq!(
            diagram_id_from_location("/editor/legacy-id", "?share=abc-123-def"),
            "abc-123-def"
        );
    }

    #[test]
    fn ut_s02_03_pathname_fallback() {
        assert_eq!(
            diagram_id_from_location("/editor/my-diagram", ""),
            "my-diagram"
        );
    }

    #[test]
    fn ut_s02_04_default_when_empty() {
        assert_eq!(diagram_id_from_location("/", ""), "default");
    }

    #[test]
    fn ut_fe_s03_01_share_route_bypasses_auth_gate() {
        assert_eq!(
            route_from_location("/editor/private-id", "?share=public-id"),
            ("public-id".to_string(), true)
        );
        assert_eq!(
            route_from_location("/editor/private-id", ""),
            ("private-id".to_string(), false)
        );
    }

    #[test]
    fn ut_s02_route_01_share_wins_and_enables_read_only_mode() {
        assert_eq!(
            route_from_location("/editor/private-id", "?share=public-id"),
            ("public-id".to_string(), true)
        );
        assert_eq!(initial_page_state("/editor/private-id", "?share=public-id"), PageState::ShareEdit);
    }

    #[test]
    fn ut_s02_404_error_is_public_and_does_not_leak_body() {
        let message = share_load_error_message(&ApiError::Server(
            404,
            "sqlite: room secret-room does not exist".to_string(),
        ));
        assert_eq!(message, "分享链接不存在或已失效");
        assert!(!message.contains("sqlite"));
        assert!(!message.contains("secret-room"));
    }

    #[test]
    fn ut_fe_s04_01_invite_route_not_treated_as_diagram_id() {
        assert_eq!(
            parse_invite_token("/invite/tok-123"),
            Some("tok-123".into())
        );
        assert_eq!(
            route_context_from_location("/invite/tok-123", ""),
            ("default".to_string(), false, Some("tok-123".to_string()))
        );
    }

    // ─── align-frontend-to-prototype — UT-FE-PROTO-01~UT-FE-PROTO-02 ────

    #[test]
    fn ut_fe_proto_01_pathname_search_returns_page_state() {
        // 默认未登录入口 → Auth
        assert_eq!(initial_page_state("/", ""), PageState::Auth);
        assert_eq!(initial_page_state("/editor", ""), PageState::Auth);
        // 未登录访问 invite → Invite（独立页）
        assert_eq!(
            initial_page_state("/invite/tok-abc", ""),
            PageState::Invite
        );
        // ?share= → ShareEdit（不要求 auth 或 rooms）
        assert_eq!(
            initial_page_state("/editor/d-uuid", "?share=public-id"),
            PageState::ShareEdit
        );
        assert_eq!(
            initial_page_state("/anything", "?share=abc"),
            PageState::ShareEdit
        );
        // 验证 parse_invite_token 不会把 invite token 当 diagram id
        assert_eq!(
            route_context_from_location("/invite/tok-123", ""),
            ("default".to_string(), false, Some("tok-123".to_string()))
        );
    }

    #[test]
    fn ut_fe_proto_02_session_notice_must_not_leak_token() {
        // 正常状态文案允许通过
        assert_eq!(
            sanitize_session_notice(Some("匿名只读分享")),
            Some("匿名只读分享".to_string())
        );
        assert_eq!(
            sanitize_session_notice(Some("会话有效")),
            Some("会话有效".to_string())
        );
        assert_eq!(
            sanitize_session_notice(Some("登录已过期，请重新登录")),
            Some("登录已过期，请重新登录".to_string())
        );
        // 含 token 原文 → 过滤为 None
        assert_eq!(sanitize_session_notice(Some("token=abc.def.ghi")), None);
        assert_eq!(
            sanitize_session_notice(Some("Bearer eyJhbGc.payload.sig")),
            None
        );
        assert_eq!(
            sanitize_session_notice(Some("refresh_token: rt-12345")),
            None
        );
        assert_eq!(
            sanitize_session_notice(Some("access_token=at-xxxx")),
            None
        );
        // 含 JWT 头 → 过滤
        assert_eq!(
            sanitize_session_notice(Some("eyJhbGciOiJIUzI1NiJ9.payload.sig")),
            None
        );
        // 空 / None → None
        assert_eq!(sanitize_session_notice(None), None);
        assert_eq!(sanitize_session_notice(Some("")), None);
        assert_eq!(sanitize_session_notice(Some("   ")), None);
    }

    #[test]
    fn ut_fe_proto_02b_auth_state_machine_transitions() {
        // 登录后从 Auth → Rooms（SIG: login_success → set_page(Rooms)）
        // 这里测试状态机语义：未登录 + default URL = Auth；
        // 登录后由前端信号从 Auth 推进到 Rooms，不依赖 URL。
        let before = initial_page_state("/", "");
        assert_eq!(before, PageState::Auth);
        // 模拟"登录后"——current_page 改为 Rooms（具体推进发生在 AppRoot）。
        // 单元测试只验证 parser 的不可变性；推进由组件层负责。
        let after: PageState = PageState::Rooms;
        assert_ne!(before, after);
    }
}
