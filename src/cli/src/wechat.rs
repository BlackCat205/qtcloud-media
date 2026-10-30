//! 微信公众号接口最小客户端：access_token、图文上传、草稿箱直出。
//! 只实现本通道用到的四个动作，不做泛化封装。

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;

const API: &str = "https://api.weixin.qq.com";

#[derive(Deserialize)]
struct ApiError {
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Serialize)]
pub struct Article {
    pub title: String,
    pub author: String,
    pub digest: String,
    pub content: String,
    pub content_source_url: String,
    pub thumb_media_id: String,
    pub need_open_comment: u8,
    pub only_fans_can_comment: u8,
}

#[derive(Serialize)]
struct DraftRequest {
    articles: Vec<Article>,
}

pub struct Wechat {
    client: reqwest::blocking::Client,
    token: String,
}

impl Wechat {
    pub fn authorize(app_id: &str, app_secret: &str) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("构建 HTTP 客户端失败")?;
        let resp: TokenResponse = client
            .get(format!("{API}/cgi-bin/token"))
            .query(&[
                ("grant_type", "client_credential"),
                ("appid", app_id),
                ("secret", app_secret),
            ])
            .send()
            .context("请求 access_token 失败（网络错误）")?
            .json()
            .context("解析 access_token 响应失败")?;
        match resp.access_token {
            Some(token) => Ok(Self { client, token }),
            None => bail!(
                "获取 access_token 失败：{} {}",
                resp.errcode.unwrap_or_default(),
                resp.errmsg.unwrap_or_default()
            ),
        }
    }

    /// 上传图文内图片，返回可在草稿 content 中直接引用的 URL。
    pub fn upload_image(&self, path: &Path) -> Result<String> {
        let bytes =
            std::fs::read(path).with_context(|| format!("读取图片失败：{}", path.display()))?;
        let part = reqwest::blocking::multipart::Part::bytes(bytes)
            .file_name(file_name(path)?)
            .mime_str(image_mime(path))?;
        let form = reqwest::blocking::multipart::Form::new().part("media", part);
        let url = format!("{API}/cgi-bin/media/uploadimg?access_token={}", self.token);
        let body = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .with_context(|| format!("上传图片失败（网络错误）：{}", path.display()))?
            .text()
            .context("读取上传图片响应失败")?;
        check_api_error(&body)?;
        let parsed: serde_json::Value =
            serde_json::from_str(&body).context("解析上传图片响应失败")?;
        let img_url = parsed
            .get("url")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .context("上传图片响应缺少 url 字段")?;
        Ok(img_url)
    }

    /// 上传封面为永久素材，返回 thumb_media_id（草稿箱封面必填项）。
    pub fn upload_thumb(&self, path: &Path) -> Result<String> {
        let bytes =
            std::fs::read(path).with_context(|| format!("读取封面图失败：{}", path.display()))?;
        let part = reqwest::blocking::multipart::Part::bytes(bytes)
            .file_name(file_name(path)?)
            .mime_str(image_mime(path))?;
        let form = reqwest::blocking::multipart::Form::new().part("media", part);
        let url = format!(
            "{API}/cgi-bin/material/add_material?access_token={}&type=image",
            self.token
        );
        let body = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .context("上传封面失败（网络错误）")?
            .text()
            .context("读取封面上传响应失败")?;
        check_api_error(&body)?;
        let parsed: serde_json::Value =
            serde_json::from_str(&body).context("解析封面上传响应失败")?;
        parsed
            .get("media_id")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .context("封面上传响应缺少 media_id 字段")
    }

    /// 新增草稿，返回草稿 media_id。文章停在草稿箱，不群发。
    ///
    /// 微信侧偶发 40007（invalid media_id）瞬时错误：素材刚上传完立即建稿
    /// 可能被拒。带退避重试三次（2s / 4s / 8s）。
    pub fn add_draft(&self, article: Article) -> Result<String> {
        let req = DraftRequest {
            articles: vec![article],
        };
        let url = format!("{API}/cgi-bin/draft/add?access_token={}", self.token);
        let backoff_secs = [2u64, 4, 8];
        let mut last_transient = String::new();
        for (attempt, wait) in backoff_secs.iter().enumerate() {
            if *wait > 0 {
                std::thread::sleep(std::time::Duration::from_secs(*wait));
            }
            let body = self
                .client
                .post(&url)
                .json(&req)
                .send()
                .context("新增草稿失败（网络错误）")?
                .text()
                .context("读取草稿响应失败")?;
            match draft_result(&body) {
                Ok(id) => return Ok(id),
                Err(DraftError::Transient(msg)) => {
                    eprintln!("建稿遇瞬时错误，重试 {}/3…（{}）", attempt + 1, msg);
                    last_transient = msg;
                }
                Err(DraftError::Fatal(msg)) => bail!("{msg}"),
            }
        }
        bail!("重试三次后建稿仍失败：{last_transient}")
    }

    /// 更新草稿箱中的同一篇草稿（接力场景：二次 publish 不新建）。
    pub fn update_draft(&self, media_id: &str, article: Article) -> Result<String> {
        let req = serde_json::json!({
            "media_id": media_id,
            "index": 0,
            "articles": [article],
        });
        let url = format!("{API}/cgi-bin/draft/update?access_token={}", self.token);
        let body = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .context("更新草稿失败（网络错误）")?
            .text()
            .context("读取草稿响应失败")?;
        check_api_error(&body)?;
        Ok(media_id.to_string())
    }
}

enum DraftError {
    /// 40007 等瞬时错误，隔片刻重试可恢复
    Transient(String),
    Fatal(String),
}

fn draft_result(body: &str) -> Result<String, DraftError> {
    if let Ok(err) = serde_json::from_str::<ApiError>(body)
        && let Some(code) = err.errcode
        && code != 0
    {
        let msg = format!(
            "微信接口返回错误：{} {}",
            code,
            err.errmsg.unwrap_or_default()
        );
        return match code {
            40007 => Err(DraftError::Transient(msg)),
            _ => Err(DraftError::Fatal(msg)),
        };
    }
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(v) => v
            .get("media_id")
            .and_then(|m| m.as_str())
            .map(str::to_owned)
            .ok_or(DraftError::Fatal("草稿响应缺少 media_id 字段".into())),
        Err(_) => Err(DraftError::Fatal("解析草稿响应失败".into())),
    }
}

fn check_api_error(body: &str) -> Result<()> {
    if let Ok(err) = serde_json::from_str::<ApiError>(body)
        && let Some(code) = err.errcode
        && code != 0
    {
        bail!(
            "微信接口返回错误：{} {}",
            code,
            err.errmsg.unwrap_or_default()
        );
    }
    Ok(())
}

fn file_name(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(str::to_owned)
        .context("非法文件名")
}

fn image_mime(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}
