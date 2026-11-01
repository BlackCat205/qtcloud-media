//! qtcloud-media provider：render / publish / 账号 / 主题 四动词薄服务。
//!
//! 定位（商业化架构，2026-10-31 拍板）：CLI 与 studio 永远免费、本地资产
//! 永远在 XDG；provider 是可收费的薄云服务——多账号托管、批量发布、
//! 发布记录审计。本地正本（草稿、主题）就在 XDG，provider 与 CLI 共用
//! 同一份 drafts/meta 与 themes 目录，服务挂了用户资产不受影响。
//!
//! 本 crate 直接依赖 qtcloud-media-cli 复用其排版引擎与微信客户端
//! （家族 provider 为 Go；此处不重写第三份排版实现，单源优先于语言惯例）。

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use qtcloud_media_cli::{drafts, paths};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    /// XDG state 根：`~/.local/state/qtcloud-media`
    pub state: PathBuf,
    /// XDG data 根：`~/.local/share/qtcloud-media`（themes 与 CLI 共用）
    pub data: PathBuf,
}

impl AppState {
    pub fn from_xdg() -> Self {
        Self {
            state: paths::state_dir(),
            data: paths::data_dir(),
        }
    }

    fn accounts_dir(&self) -> PathBuf {
        self.state.join("provider/accounts")
    }

    fn records_path(&self) -> PathBuf {
        self.state.join("provider/records.jsonl")
    }

    fn themes_dir(&self) -> PathBuf {
        self.data.join("themes")
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/render", post(render_md))
        .route("/publish", post(publish))
        .route("/accounts", get(list_accounts))
        .route("/accounts/{name}", put(put_account))
        .route("/themes", get(list_themes))
        .route("/themes/{name}", get(get_theme).put(put_theme))
        .route("/records", get(list_records))
        .route("/articles", get(list_articles))
        .route("/articles/{id}", get(get_article))
        .with_state(Arc::new(state))
}

/// anyhow 错误 → HTTP 400/500 的适配层。
pub struct ApiError(anyhow::Error);

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self(e)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        Self(e.into())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self(e.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, format!("{}", self.0)).into_response()
    }
}

// ── 账号 ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub app_id: String,
    pub app_secret: String,
}

fn account_path(state: &AppState, name: &str) -> Result<PathBuf> {
    check_name(name)?;
    Ok(state.accounts_dir().join(format!("{name}.json")))
}

/// 名称只允许中英数字与连字符，防路径穿越。
fn check_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        bail!("非法名称「{name}」：只允许中英文字母、数字、连字符、下划线，最长 64");
    }
    Ok(())
}

async fn put_account(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(account): Json<Account>,
) -> Result<impl IntoResponse, ApiError> {
    let path = account_path(&state, &name)?;
    std::fs::create_dir_all(state.accounts_dir())
        .with_context(|| format!("创建账号目录失败：{}", state.accounts_dir().display()))?;
    std::fs::write(&path, serde_json::to_string_pretty(&account)?)
        .with_context(|| format!("写入账号失败：{}", path.display()))?;
    set_owner_only(&path)?;
    Ok((
        StatusCode::CREATED,
        format!("账号「{name}」已保存（本地 {}）", path.display()),
    ))
}

async fn list_accounts(State(state): State<Arc<AppState>>) -> Result<Json<Vec<String>>, ApiError> {
    let dir = state.accounts_dir();
    if !dir.is_dir() {
        return Ok(Json(vec![]));
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter(|f| f.ends_with(".json"))
        .map(|f| f.trim_end_matches(".json").to_string())
        .collect();
    names.sort();
    Ok(Json(names))
}

#[cfg(unix)]
fn set_owner_only(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_owner_only(_path: &std::path::Path) -> Result<()> {
    Ok(())
}

// ── 主题（与 CLI 共用 XDG data 目录）────────────────────────────────

async fn list_themes(State(state): State<Arc<AppState>>) -> Result<Json<Vec<String>>, ApiError> {
    let dir = state.themes_dir();
    if !dir.is_dir() {
        return Ok(Json(vec![]));
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter(|f| f.ends_with(".toml"))
        .map(|f| f.trim_end_matches(".toml").to_string())
        .collect();
    names.sort();
    Ok(Json(names))
}

async fn get_theme(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Response, ApiError> {
    check_name(&name)?;
    let path = state.themes_dir().join(format!("{name}.toml"));
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("主题不存在：{}", path.display()))?;
    Ok(raw.into_response())
}

async fn put_theme(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    body: String,
) -> Result<impl IntoResponse, ApiError> {
    check_name(&name)?;
    // 与 CLI 同一字段集的扁平 TOML；先按 CLI 的解析器校验再落盘
    toml::from_str::<toml::Table>(&body).context("主题不是合法 TOML")?;
    let dir = state.themes_dir();
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("创建主题目录失败：{}", dir.display()))?;
    let path = dir.join(format!("{name}.toml"));
    std::fs::write(&path, &body).with_context(|| format!("写入失败：{}", path.display()))?;
    Ok(format!(
        "主题「{name}」已保存（与 CLI 共用 {}）",
        path.display()
    ))
}

// ── render ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RenderReq {
    pub markdown: String,
    #[serde(default)]
    pub theme: Option<String>,
}

async fn render_md(Json(req): Json<RenderReq>) -> Result<Json<serde_json::Value>, ApiError> {
    let html = tokio::task::spawn_blocking(move || -> Result<String> {
        let theme_file = match &req.theme {
            Some(spec) => Some(paths::resolve_theme(spec).with_context(|| {
                format!(
                    "找不到主题「{spec}」（查 {} 与 {} 下的 themes/）",
                    paths::data_dir().display(),
                    paths::config_dir().display()
                )
            })?),
            None => None,
        };
        let theme = match &theme_file {
            Some(p) => qtcloud_media_cli::theme::Theme::load(p)?,
            None => qtcloud_media_cli::theme::Theme::default(),
        };
        Ok(qtcloud_media_cli::render::render(&req.markdown, &theme))
    })
    .await
    .context("render 任务失败")??;
    Ok(Json(serde_json::json!({ "html": html })))
}

// ── publish ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PublishReq {
    /// 账号名（GET /accounts 列出的名字）
    pub account: String,
    pub markdown: String,
    #[serde(default)]
    pub theme: Option<String>,
    pub title: String,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub source_url: String,
    /// 封面三选一：已有 media_id / base64 图片数据 / 图片 URL；都空则要求 draft 元数据已有封面
    #[serde(default)]
    pub thumb_media_id: Option<String>,
    #[serde(default)]
    pub thumb_data_b64: Option<String>,
    #[serde(default)]
    pub thumb_url: Option<String>,
    /// 草稿名（可选）：给定则走 XDG 草稿接力——带 media_id 更新同一篇
    #[serde(default)]
    pub draft: Option<String>,
}

#[derive(Serialize)]
pub struct PublishResult {
    pub media_id: String,
    /// "已推入"（新建）或"已更新"（接力）
    pub action: String,
}

async fn publish(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PublishReq>,
) -> Result<Json<PublishResult>, ApiError> {
    let account = load_account(&state, &req.account)?;
    let state_inner = (*state).clone();
    let result = tokio::task::spawn_blocking(move || publish_blocking(state_inner, account, req))
        .await
        .context("publish 任务失败")??;
    Ok(Json(result))
}

fn load_account(state: &AppState, name: &str) -> Result<Account> {
    let path = account_path(state, name)?;
    let raw = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "账号「{name}」不存在（{}），先 PUT /accounts/{name}",
            path.display()
        )
    })?;
    Ok(serde_json::from_str(&raw)?)
}

fn publish_blocking(state: AppState, account: Account, req: PublishReq) -> Result<PublishResult> {
    use qtcloud_media_cli::wechat::{Article, Wechat};

    let wechat = Wechat::authorize(&account.app_id, &account.app_secret)?;

    // 封面：media_id 直用；base64/URL 现场上传
    let thumb_media_id = if let Some(id) = req.thumb_media_id.filter(|s| !s.is_empty()) {
        id
    } else if let Some(b64) = req.thumb_data_b64.filter(|s| !s.is_empty()) {
        let bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            b64.trim_end_matches('\n'),
        )
        .context("thumb_data_b64 解码失败")?;
        let (ext, mime) = sniff_image(&bytes);
        wechat.upload_thumb_bytes(&bytes, &format!("cover.{ext}"), mime)?
    } else if let Some(url) = req.thumb_url.filter(|s| !s.is_empty()) {
        let bytes = reqwest_get_bytes(&url)?;
        let name = url
            .rsplit('/')
            .next()
            .unwrap_or("cover.jpg")
            .split('?')
            .next()
            .unwrap_or("cover.jpg");
        let mime = match name
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            _ => "image/jpeg",
        };
        wechat.upload_thumb_bytes(&bytes, name, mime)?
    } else if let Some(name) = &req.draft {
        drafts::load_meta(name).thumb
    } else {
        bail!("缺少封面：thumb_media_id / thumb_data_b64 / thumb_url 三选一")
    };

    // 草稿接力时先读旧 meta（可能带 media_id）
    let existing = req
        .draft
        .as_deref()
        .map(drafts::load_meta)
        .unwrap_or_default();

    let theme_file = match &req.theme {
        Some(spec) => {
            Some(paths::resolve_theme(spec).with_context(|| format!("找不到主题「{spec}\""))?)
        }
        None => None,
    };
    let theme = match &theme_file {
        Some(p) => qtcloud_media_cli::theme::Theme::load(p)?,
        None => qtcloud_media_cli::theme::Theme::default(),
    };
    let html = qtcloud_media_cli::render::render(&req.markdown, &theme);

    let article = Article {
        title: req.title.clone(),
        author: req.author,
        digest: req.digest.clone(),
        content: html,
        content_source_url: req.source_url,
        thumb_media_id: thumb_media_id.clone(),
        need_open_comment: 0,
        only_fans_can_comment: 0,
    };
    let (media_id, action) = if existing.media_id.is_empty() {
        (wechat.add_draft(article)?, "已推入")
    } else {
        (wechat.update_draft(&existing.media_id, article)?, "已更新")
    };

    // 接力 meta 与发布记录（本地正本）
    if let Some(name) = &req.draft {
        drafts::save_meta(
            name,
            &drafts::DraftMeta {
                title: req.title.clone(),
                digest: req.digest.clone(),
                thumb: thumb_media_id.clone(),
                theme: req.theme.clone().unwrap_or_default(),
                media_id: media_id.clone(),
            },
        )?;
    }
    append_record(&state, &req.account, &req.title, &media_id, action)?;

    Ok(PublishResult {
        media_id,
        action: action.to_string(),
    })
}

fn reqwest_get_bytes(url: &str) -> Result<Vec<u8>> {
    let resp = reqwest::blocking::get(url).with_context(|| format!("下载封面失败：{url}"))?;
    if !resp.status().is_success() {
        bail!("下载封面失败：{url} → {}", resp.status());
    }
    Ok(resp.bytes()?.to_vec())
}

/// 按魔数字节嗅探图片类型（base64 封面没有文件名可依）。
fn sniff_image(bytes: &[u8]) -> (&'static str, &'static str) {
    match bytes {
        [0x89, b'P', b'N', b'G', ..] => ("png", "image/png"),
        [0xFF, 0xD8, 0xFF, ..] => ("jpg", "image/jpeg"),
        [b'G', b'I', b'F', b'8', ..] => ("gif", "image/gif"),
        [b'R', b'I', b'F', b'F', .., b'W', b'E', b'B', b'P'] => ("webp", "image/webp"),
        _ => ("jpg", "image/jpeg"),
    }
}

fn append_record(
    state: &AppState,
    account: &str,
    title: &str,
    media_id: &str,
    action: &str,
) -> Result<()> {
    let path = state.records_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let record = serde_json::json!({
        "ts": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        "account": account,
        "title": title,
        "media_id": media_id,
        "action": action,
    });
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(f, "{}", serde_json::to_string(&record)?)?;
    Ok(())
}

async fn list_records(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let raw = std::fs::read_to_string(state.records_path()).unwrap_or_default();
    let records = raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    Ok(Json(records))
}

// ── 历史文章（微信 freepublish 发布能力）──────────────────────────

#[derive(Deserialize)]
pub struct AccountQuery {
    pub account: String,
}

#[derive(Deserialize)]
pub struct ArticleQuery {
    pub account: String,
    /// 通道：draft（缺省）/ publish；draft 时 id 为 media_id，publish 时为 article_id
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Deserialize)]
pub struct ArticlesQuery {
    pub account: String,
    /// 通道：draft＝草稿箱（缺省，历史文章的草稿正本），publish＝已发布（需账号有发布能力）
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_count")]
    pub count: u32,
}

fn default_count() -> u32 {
    10
}

/// 按通道取历史文章：draft 走草稿箱，publish 走 freepublish（未授权账号报 48001）。
fn fetch_history(
    account: &Account,
    source: Option<&str>,
    offset: u32,
    count: u32,
) -> Result<(Vec<qtcloud_media_cli::wechat::PubArticle>, u64)> {
    use qtcloud_media_cli::wechat::Wechat;

    let wechat = Wechat::authorize(&account.app_id, &account.app_secret)?;
    match source.unwrap_or("draft") {
        "publish" => wechat.freepublish_list(offset, count),
        _ => wechat.draft_list(offset, count),
    }
}

#[derive(Serialize)]
pub struct ArticleSummary {
    pub article_id: String,
    pub title: String,
    pub digest: String,
    pub publish_time: u64,
    pub url: String,
}

async fn list_articles(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ArticlesQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let account = load_account(&state, &q.account)?;
    let source = q.source.clone();
    let (items, total) = tokio::task::spawn_blocking(move || {
        fetch_history(&account, source.as_deref(), q.offset, q.count)
    })
    .await
    .context("history 任务失败")??;
    let items: Vec<ArticleSummary> = items
        .into_iter()
        .map(|a| ArticleSummary {
            article_id: a.article_id,
            title: a.title,
            digest: a.digest,
            publish_time: a.publish_time,
            url: a.url,
        })
        .collect();
    Ok(Json(serde_json::json!({ "total": total, "items": items })))
}

async fn get_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(q): Query<ArticleQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let account = load_account(&state, &q.account)?;
    let article = tokio::task::spawn_blocking(move || {
        use qtcloud_media_cli::wechat::Wechat;

        let wechat = Wechat::authorize(&account.app_id, &account.app_secret)?;
        match q.source.as_deref().unwrap_or("draft") {
            "publish" => wechat.freepublish_get(&id),
            _ => wechat.draft_get(&id),
        }
    })
    .await
    .context("history 任务失败")??;
    Ok(Json(serde_json::to_value(article)?))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "qtcloud-media-provider",
        "xdg": {
            "state": paths::state_dir().display().to_string(),
            "data": paths::data_dir().display().to_string(),
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn setup() -> (TempDir, AppState) {
        let tmp = tempfile::tempdir().unwrap();
        let state = AppState {
            state: tmp.path().join("state"),
            data: tmp.path().join("data"),
        };
        (tmp, state)
    }

    async fn send_json(app: &Router, method: &str, uri: &str, body: &str) -> Response {
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        app.clone().oneshot(req).await.unwrap()
    }

    #[tokio::test]
    async fn health_reports_xdg() {
        let (_tmp, state) = setup();
        let resp = router(state)
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn account_put_then_list() {
        let (_tmp, state) = setup();
        let app = router(state);
        let resp = send_json(
            &app,
            "PUT",
            "/accounts/main",
            r#"{"app_id":"wx123","app_secret":"s"}"#,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/accounts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<Vec<String>>(&bytes).unwrap(),
            vec!["main"]
        );
    }

    #[tokio::test]
    async fn account_rejects_path_traversal() {
        let (_tmp, state) = setup();
        let app = router(state);
        let resp = send_json(
            &app,
            "PUT",
            "/accounts/..%2Fetc",
            r#"{"app_id":"a","app_secret":"b"}"#,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn theme_put_get_list() {
        let (_tmp, state) = setup();
        let app = router(state);
        let req = Request::builder()
            .method("PUT")
            .uri("/themes/brand")
            .body(Body::from("body_color = \"#333333\"\n"))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/themes/brand")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn render_endpoint_produces_html() {
        let (_tmp, state) = setup();
        let app = router(state);
        let resp = send_json(
            &app,
            "POST",
            "/render",
            r##"{"markdown":"# 标题\n\n正文段落"}"##,
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), 65536).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let html = v["html"].as_str().unwrap();
        assert!(html.contains("正文段落"));
        assert!(!html.contains("<h1"), "首段 H1 不应进正文（公众号规则）");
    }

    #[tokio::test]
    async fn articles_require_account() {
        let (_tmp, state) = setup();
        let app = router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/articles?account=missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn records_append_then_list() {
        let (_tmp, state) = setup();
        append_record(&state, "main", "标题", "MID1", "已推入").unwrap();
        append_record(&state, "main", "标题2", "MID2", "已更新").unwrap();
        let app = router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/records")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(resp.into_body(), 65536).await.unwrap();
        let v: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[1]["action"], "已更新");
    }

    #[test]
    fn sniffs_image_magic_bytes() {
        assert_eq!(
            sniff_image(&[0x89, b'P', b'N', b'G', 0, 0]),
            ("png", "image/png")
        );
        assert_eq!(
            sniff_image(&[0xFF, 0xD8, 0xFF, 0xE0]),
            ("jpg", "image/jpeg")
        );
        assert_eq!(sniff_image(b"GIF89a.."), ("gif", "image/gif"));
        assert_eq!(sniff_image(b"RIFF\0\0\0\0WEBP"), ("webp", "image/webp"));
        assert_eq!(sniff_image(b"unknown"), ("jpg", "image/jpeg"));
    }
}
