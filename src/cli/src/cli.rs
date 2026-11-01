//! 命令行入口：render（只排版）、publish（排版 + 推草稿箱）、skills（技能管理）。

use crate::drafts::{self, DraftMeta};
use crate::render;
use crate::theme::Theme;
use crate::wechat::{Article, Wechat};
use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

const ENV_APP_ID: &str = "QTCLOUD_MEDIA_APP_ID";
const ENV_APP_SECRET: &str = "QTCLOUD_MEDIA_APP_SECRET";

const WECHAT_SKILL: &str = include_str!("../skills/qtcloud-media-wechat/SKILL.md");

#[derive(Parser)]
#[command(
    name = "qtcloud-media",
    about = "量潮媒体云命令行工具：公众号排版与草稿箱直出",
    version
)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Markdown → 公众号草稿 HTML（只排版，不联网）
    Render {
        /// Markdown 文件或草稿名
        markdown: String,
        /// 主题：文件路径或名称（名称按 XDG 查 `<data|config>/themes/<name>.toml`），缺省用内置默认主题
        #[arg(long)]
        theme: Option<String>,
        /// 输出 HTML 文件，缺省打印到标准输出
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Markdown → 排版 → 公众号草稿箱（不群发；草稿名再次 publish 走更新）
    Publish {
        /// Markdown 文件或草稿名
        markdown: String,
        /// 主题：文件路径或名称（名称按 XDG 查 `<data|config>/themes/<name>.toml`），缺省用内置默认主题
        #[arg(long)]
        theme: Option<String>,
        /// 文章标题，缺省按标题规则从正文解析（YAML title / 唯一一级标题）
        #[arg(long)]
        title: Option<String>,
        /// 封面：本地图片路径或已有 thumb_media_id
        #[arg(long)]
        thumb: String,
        /// 摘要
        #[arg(long, default_value = "")]
        digest: String,
        /// 作者
        #[arg(long, default_value = "")]
        author: String,
        /// 原文链接
        #[arg(long, default_value = "")]
        source_url: String,
        /// 远程模式：经 provider 发布（账号托管在 provider 侧，需 --account）
        #[arg(long)]
        remote: bool,
        /// provider 地址，缺省取环境变量 QTCLOUD_MEDIA_PROVIDER，再缺省 http://127.0.0.1:8787
        #[arg(long)]
        provider: Option<String>,
        /// provider 侧托管的账号名（--remote 时必填）
        #[arg(long)]
        account: Option<String>,
    },
    /// 历史文章：从公众号拉取已发布文章（经 provider，账号托管在服务端）
    History {
        #[command(subcommand)]
        cmd: HistoryCmd,
    },
    /// 技能管理：列出 / 安装内嵌的 agent 技能文件
    Skills {
        #[command(subcommand)]
        cmd: SkillsCmd,
    },
    /// 草稿管理：CLI 与 studio 共用的接力工作区（XDG state）
    Draft {
        #[command(subcommand)]
        cmd: DraftCmd,
    },
}

#[derive(Subcommand)]
enum DraftCmd {
    /// 列出草稿（名称、标题、是否已推草稿箱）
    List,
    /// 新建草稿（生成正文骨架，可在 CLI 或 studio 中继续写）
    New { name: String },
    /// 查看草稿：正文路径与 meta 状态
    Show { name: String },
}

#[derive(Subcommand)]
enum HistoryCmd {
    /// 列出已发布文章（发布时间、标题、article_id）
    List {
        /// provider 地址，缺省取环境变量 QTCLOUD_MEDIA_PROVIDER，再缺省 http://127.0.0.1:8787
        #[arg(long)]
        provider: Option<String>,
        /// provider 侧托管的账号名
        #[arg(long)]
        account: Option<String>,
        /// 通道：draft＝草稿箱（缺省，历史文章的草稿正本），publish＝已发布（需账号有发布能力）
        #[arg(long)]
        source: Option<String>,
        /// 本页条数
        #[arg(long, default_value_t = 10)]
        count: u32,
        /// 分页偏移
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
    /// 拉取一篇历史文章为 Markdown（标题写入 YAML front matter，正文按标题规则拆好）
    Get {
        /// 文章 ID（history list 输出）
        article_id: String,
        /// 输出 Markdown 文件，缺省 <article_id>.md
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// provider 地址，缺省取环境变量 QTCLOUD_MEDIA_PROVIDER，再缺省 http://127.0.0.1:8787
        #[arg(long)]
        provider: Option<String>,
        /// provider 侧托管的账号名
        #[arg(long)]
        account: Option<String>,
        /// 通道：draft＝草稿箱（缺省），publish＝已发布；draft 时 id 为 media_id，publish 时为 article_id
        #[arg(long)]
        source: Option<String>,
    },
}

#[derive(Subcommand)]
enum SkillsCmd {
    /// 列出内嵌技能
    List,
    /// 安装技能到本机 agent 技能目录（缺省 ~/.agents/skills）
    Install {
        /// 技能名，缺省安装全部
        name: Option<String>,
        /// 目标目录，缺省 ~/.agents/skills
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

pub fn run() -> Result<()> {
    match Args::parse().cmd {
        Cmd::Render {
            markdown,
            theme,
            output,
        } => {
            let input = drafts::resolve_input(&markdown)?;
            let html = render_file(input.path(), theme.as_deref())?;
            match output {
                Some(path) => {
                    std::fs::write(&path, &html)
                        .with_context(|| format!("写入失败：{}", path.display()))?;
                    eprintln!("已写入 {}", path.display());
                }
                None => println!("{html}"),
            }
            Ok(())
        }
        Cmd::Publish {
            markdown,
            theme,
            title,
            thumb,
            digest,
            author,
            source_url,
            remote,
            provider,
            account,
        } => {
            let input = drafts::resolve_input(&markdown)?;
            // 标题：--title 显式给出优先，否则按标题规则从正文解析
            let title = match title {
                Some(t) => t,
                None => {
                    let text = std::fs::read_to_string(input.path())
                        .with_context(|| format!("读取正文失败：{}", input.path().display()))?;
                    crate::render::split_title(&text).0.context(
                        "正文解析不出标题：用 --title 明确给出，或正文含唯一一行一级标题 / YAML front matter 的 title:",
                    )?
                }
            };
            if remote {
                return publish_remote(
                    &input,
                    theme.as_deref(),
                    &title,
                    &thumb,
                    &digest,
                    &author,
                    &source_url,
                    account.as_deref(),
                    provider.as_deref(),
                );
            }
            let html = render_file(input.path(), theme.as_deref())?;
            let wechat = Wechat::authorize(&env_var(ENV_APP_ID)?, &env_var(ENV_APP_SECRET)?)?;
            let html = inline_local_images(&html, input.path(), &wechat)?;
            let thumb_media_id = if thumb.starts_with("http") || Path::new(&thumb).is_file() {
                wechat.upload_thumb(Path::new(&thumb))?
            } else {
                thumb.clone()
            };
            let article = Article {
                title: title.clone(),
                author,
                digest: digest.clone(),
                content: html,
                content_source_url: source_url,
                thumb_media_id,
                need_open_comment: 0,
                only_fans_can_comment: 0,
            };
            // 接力：草稿带 media_id 时更新同一篇，不新建
            let existing = match &input {
                drafts::Input::Draft { name, .. } => drafts::load_meta(name).media_id,
                drafts::Input::File(_) => String::new(),
            };
            let (draft_id, action) = if existing.is_empty() {
                (wechat.add_draft(article)?, "已推入")
            } else {
                (wechat.update_draft(&existing, article)?, "已更新")
            };
            println!("{action}公众号草稿箱，media_id={draft_id}");
            println!("注意：草稿不会自动群发，请到公众号后台确认后手动群发。");
            if let drafts::Input::Draft { name, .. } = &input {
                drafts::save_meta(
                    name,
                    &DraftMeta {
                        title,
                        digest,
                        thumb,
                        theme: theme.unwrap_or_default(),
                        media_id: draft_id.clone(),
                    },
                )?;
                println!("草稿状态已保存：{}", drafts::meta_path(name).display());
            }
            Ok(())
        }
        Cmd::History { cmd } => match cmd {
            HistoryCmd::List {
                provider,
                account,
                source,
                count,
                offset,
            } => {
                let base = provider_base(provider.as_deref());
                let account = resolve_account(account.as_deref())?;
                let source = source.unwrap_or_else(|| "draft".to_string());
                let client = reqwest::blocking::Client::new();
                let resp = client
                    .get(format!("{base}/articles"))
                    .query(&[
                        ("account", account.as_str()),
                        ("source", source.as_str()),
                        ("offset", &offset.to_string()),
                        ("count", &count.to_string()),
                    ])
                    .send()
                    .with_context(|| format!("调用 provider 失败：{base}（服务起来了吗？）"))?;
                if !resp.status().is_success() {
                    bail!("provider 返回错误：{}", resp.text().unwrap_or_default());
                }
                let v: serde_json::Value = resp.json().context("解析 provider 响应失败")?;
                let total = v["total"].as_u64().unwrap_or_default();
                let items = v["items"].as_array().cloned().unwrap_or_default();
                println!(
                    "共 {total} 篇（本页 {}，offset {offset}，账号 {account}）",
                    items.len()
                );
                for it in &items {
                    println!(
                        "{}\t{}\t{}",
                        it["publish_time"].as_u64().unwrap_or_default(),
                        it["title"].as_str().unwrap_or(""),
                        it["article_id"].as_str().unwrap_or("")
                    );
                }
                Ok(())
            }
            HistoryCmd::Get {
                article_id,
                output,
                provider,
                account,
                source,
            } => {
                let base = provider_base(provider.as_deref());
                let account = resolve_account(account.as_deref())?;
                let source = source.unwrap_or_else(|| "draft".to_string());
                let client = reqwest::blocking::Client::new();
                let resp = client
                    .get(format!("{base}/articles/{article_id}"))
                    .query(&[("account", account.as_str()), ("source", source.as_str())])
                    .send()
                    .with_context(|| format!("调用 provider 失败：{base}（服务起来了吗？）"))?;
                if !resp.status().is_success() {
                    bail!("provider 返回错误：{}", resp.text().unwrap_or_default());
                }
                let v: serde_json::Value = resp.json().context("解析 provider 响应失败")?;
                let title = v["title"].as_str().unwrap_or_default().to_string();
                let content = v["content"].as_str().unwrap_or_default();
                let body = crate::htmlmd::html_to_markdown(content);
                let markdown = format!("---\ntitle: {title}\n---\n\n{body}");
                let out = output.unwrap_or_else(|| PathBuf::from(format!("{article_id}.md")));
                std::fs::write(&out, &markdown)
                    .with_context(|| format!("写入失败：{}", out.display()))?;
                println!("已写入 {}", out.display());
                println!("标题：{title}");
                if let Some(d) = v["digest"].as_str().filter(|s| !s.is_empty()) {
                    println!("摘要：{d}");
                }
                if let Some(t) = v["thumb_url"].as_str().filter(|s| !s.is_empty()) {
                    println!("封面 URL：{t}");
                }
                if let Some(t) = v["thumb_media_id"].as_str().filter(|s| !s.is_empty()) {
                    println!("封面 media_id：{t}（publish --thumb 直接用）");
                }
                if let Some(u) = v["url"].as_str().filter(|s| !s.is_empty()) {
                    println!("原文：{u}");
                }
                println!(
                    "改完后 publish 可直接用此文件（标题走 YAML front matter）。\n注意：文中图片仍是原文链接，如需入库需重传。"
                );
                Ok(())
            }
        },
        Cmd::Skills { cmd } => match cmd {
            SkillsCmd::List => {
                for skill in embedded_skills() {
                    println!("{}\n  {}", skill.name, skill.description);
                }
                Ok(())
            }
            SkillsCmd::Install { name, dir } => {
                let dir = match dir {
                    Some(d) => d,
                    None => std::env::home_dir()
                        .context("无法确定用户主目录")?
                        .join(".agents/skills"),
                };
                let mut installed = 0;
                for skill in embedded_skills() {
                    if let Some(n) = &name
                        && *n != skill.name
                    {
                        continue;
                    }
                    let config = agent_skills_rs::InstallConfig {
                        mode: agent_skills_rs::InstallMode::Copy,
                        ..agent_skills_rs::InstallConfig::new(dir.clone())
                    };
                    let result = agent_skills_rs::install_skill(&skill, &config)
                        .with_context(|| format!("安装技能失败：{}", skill.name))?;
                    println!("已安装 {} → {}", skill.name, result.path.display());
                    installed += 1;
                }
                if installed == 0 {
                    bail!("没有匹配的技能：{}", name.unwrap_or_default());
                }
                Ok(())
            }
        },
        Cmd::Draft { cmd } => match cmd {
            DraftCmd::List => {
                let names = drafts::list()?;
                if names.is_empty() {
                    println!("（暂无草稿，目录 {}）", drafts::drafts_dir().display());
                }
                for name in names {
                    let meta = drafts::load_meta(&name);
                    let state = if meta.media_id.is_empty() {
                        "未推草稿箱"
                    } else {
                        "已推草稿箱"
                    };
                    println!("{name}\t{state}\t{}", meta.title);
                }
                Ok(())
            }
            DraftCmd::New { name } => {
                let path = drafts::markdown_path(&name);
                if path.exists() {
                    bail!("草稿已存在：{}", path.display());
                }
                std::fs::create_dir_all(drafts::drafts_dir()).with_context(|| {
                    format!("创建草稿目录失败：{}", drafts::drafts_dir().display())
                })?;
                std::fs::write(&path, format!("# {name}\n\n"))?;
                println!("已新建草稿：{}", path.display());
                println!("在 CLI 或 studio 里继续写，publish 时用草稿名「{name}」。");
                Ok(())
            }
            DraftCmd::Show { name } => {
                let path = drafts::markdown_path(&name);
                if !path.exists() {
                    bail!("草稿不存在：{}", path.display());
                }
                let meta = drafts::load_meta(&name);
                println!("正文：{}", path.display());
                println!("标题：{}", meta.title);
                println!("摘要：{}", meta.digest);
                println!(
                    "主题：{}",
                    if meta.theme.is_empty() {
                        "（内置默认）"
                    } else {
                        &meta.theme
                    }
                );
                println!("封面：{}", meta.thumb);
                println!(
                    "草稿箱 media_id：{}",
                    if meta.media_id.is_empty() {
                        "（未推过）"
                    } else {
                        &meta.media_id
                    }
                );
                Ok(())
            }
        },
    }
}

/// 远程模式：把发布请求交给 provider（账号托管、发布记录都在服务端；
/// 本地正本不变——草稿与 meta 仍是 XDG 同一套）。
// CLI 参数透传函数，参数多是形态使然
#[allow(clippy::too_many_arguments)]
fn publish_remote(
    input: &drafts::Input,
    theme: Option<&str>,
    title: &str,
    thumb: &str,
    digest: &str,
    author: &str,
    source_url: &str,
    account: Option<&str>,
    provider: Option<&str>,
) -> Result<()> {
    use base64::Engine as _;

    let account = resolve_account(account)?;
    let base = provider_base(provider);
    let markdown = std::fs::read_to_string(input.path())
        .with_context(|| format!("读取正文失败：{}", input.path().display()))?;

    let mut body = serde_json::json!({
        "account": account,
        "markdown": markdown,
        "title": title,
        "digest": digest,
        "author": author,
        "source_url": source_url,
    });
    if let Some(t) = theme {
        body["theme"] = t.into();
    }
    if let drafts::Input::Draft { name, .. } = input {
        body["draft"] = name.clone().into();
    }
    // 封面三选一：URL → thumb_url；本地文件 → base64（provider 按魔数嗅探类型）；否则按已有 media_id
    if thumb.starts_with("http") {
        body["thumb_url"] = thumb.into();
    } else if Path::new(thumb).is_file() {
        let bytes = std::fs::read(thumb).with_context(|| format!("读取封面图失败：{thumb}"))?;
        body["thumb_data_b64"] = base64::engine::general_purpose::STANDARD
            .encode(&bytes)
            .into();
    } else {
        body["thumb_media_id"] = thumb.into();
    }

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{base}/publish"))
        .json(&body)
        .send()
        .with_context(|| format!("调用 provider 失败：{base}（服务起来了吗？）"))?;
    if !resp.status().is_success() {
        let text = resp.text().unwrap_or_default();
        bail!("provider 返回错误：{text}");
    }
    let v: serde_json::Value = resp.json().context("解析 provider 响应失败")?;
    println!(
        "{}公众号草稿箱，media_id={}（provider {}，账号 {account}）",
        v["action"].as_str().unwrap_or("已推送"),
        v["media_id"].as_str().unwrap_or("?"),
        base
    );
    println!("注意：草稿不会自动群发，请到公众号后台确认后手动群发。");
    Ok(())
}

fn embedded_skills() -> Vec<agent_skills_rs::Skill> {
    vec![
        agent_skills_rs::register_embedded_skill(WECHAT_SKILL, &[])
            .expect("内嵌技能 frontmatter 必须含 name 与 description"),
    ]
}

pub fn render_file(markdown: &Path, theme_spec: Option<&str>) -> Result<String> {
    let text = std::fs::read_to_string(markdown)
        .with_context(|| format!("读取 Markdown 失败：{}", markdown.display()))?;
    let theme = match theme_spec {
        Some(spec) => match crate::paths::resolve_theme(spec) {
            Some(path) => Theme::load(&path)?,
            None => bail!(
                "找不到主题「{spec}」：既不是文件，也不在 {} 或 {} 的 themes/ 下",
                crate::paths::data_dir().display(),
                crate::paths::config_dir().display()
            ),
        },
        None => Theme::default(),
    };
    Ok(render::render(&text, &theme))
}

/// 把 content 里的本地图片替换为已上传的微信 URL。
pub fn inline_local_images(html: &str, markdown: &Path, wechat: &Wechat) -> Result<String> {
    let re = regex::Regex::new(r#"<img[^>]*src="([^"]+)""#).expect("静态正则");
    let base = markdown.parent().unwrap_or_else(|| Path::new("."));
    let mut uploads: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for cap in re.captures_iter(html) {
        let src = &cap[1];
        if src.starts_with("http") || uploads.contains_key(src) {
            continue;
        }
        let path = base.join(src);
        let url = wechat.upload_image(&path)?;
        uploads.insert(src.to_string(), url);
    }
    Ok(re
        .replace_all(html, |caps: &regex::Captures| {
            let src = &caps[1];
            match uploads.get(src) {
                Some(url) => caps[0].replace(src, url),
                None => caps[0].to_string(),
            }
        })
        .into_owned())
}

fn env_var(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("缺少环境变量 {name}"))
}

/// provider 地址：--provider > 环境变量 > 缺省本机。
fn provider_base(provider: Option<&str>) -> String {
    provider
        .map(str::to_owned)
        .or_else(|| std::env::var("QTCLOUD_MEDIA_PROVIDER").ok())
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string())
}

/// provider 侧托管的账号名。
fn resolve_account(account: Option<&str>) -> Result<String> {
    account
        .map(str::to_owned)
        .context("需要 --account <provider 侧账号名>（先把账号 PUT 给 provider）")
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_skill_parses() {
        let skills = embedded_skills();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "qtcloud-media-wechat");
        assert!(!skills[0].description.is_empty());
    }
}
