//! 命令行入口。
//!
//! 命令树按「管理单位 × 动作」组织：单位是名词（article 文章 / note 图文），
//! 动作是动词（render/push/pull/draft…），格式与平台是动作的参数——
//! 导出某种格式（html、png…）或连接某个具体平台（wechat…，未来可扩展）。

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
    about = "量潮媒体云命令行工具：文章与图文的排版、导出与平台推送",
    version
)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 文章：导出格式、连接平台、取回、工作区
    Article {
        #[command(subcommand)]
        cmd: ArticleCmd,
    },
    /// 图文：导出格式（小红书图集等）
    Note {
        #[command(subcommand)]
        cmd: NoteCmd,
    },
    /// 连接设置：provider 与账号，配一次各命令共用
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    /// 技能管理：列出 / 安装内嵌的 agent 技能文件
    Skills {
        #[command(subcommand)]
        cmd: SkillsCmd,
    },
}

#[derive(Subcommand)]
enum ArticleCmd {
    /// 导出格式 → 公众号草稿 HTML（只排版，不联网）
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
    /// 连接平台 → wechat 公众号草稿箱（不群发；草稿名再次 push 走更新）
    Push {
        /// Markdown 文件或草稿名
        markdown: String,
        /// 平台，目前只支持 wechat
        #[arg(long, default_value = "wechat")]
        platform: String,
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
        /// 远程模式：经 provider 推送（账号托管在 provider 侧）
        #[arg(long)]
        remote: bool,
        /// provider 地址覆盖（缺省：环境变量 > `config set provider` > 本机 8787）
        #[arg(long)]
        provider: Option<String>,
        /// 账号名覆盖（缺省：环境变量 > `config set account`）
        #[arg(long)]
        account: Option<String>,
    },
    /// 从平台取回（经 provider；缺省通道＝草稿箱，publish 通道需账号有发布能力）
    Pull {
        #[command(subcommand)]
        cmd: PullCmd,
    },
    /// 文章工作区：CLI 与 studio 共用的草稿接力区（XDG state）
    Draft {
        #[command(subcommand)]
        cmd: DraftCmd,
    },
}

#[derive(Subcommand)]
enum PullCmd {
    /// 列出平台文章（时间、标题、id）
    List {
        /// 平台，目前只支持 wechat
        #[arg(long, default_value = "wechat")]
        platform: String,
        /// 账号名覆盖（缺省：环境变量 > `config set account`）
        #[arg(long)]
        account: Option<String>,
        /// provider 地址覆盖（缺省：环境变量 > `config set provider` > 本机 8787）
        #[arg(long)]
        provider: Option<String>,
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
    /// 取回一篇为 Markdown（标题写入 YAML front matter，正文按标题规则拆好）
    Get {
        /// 文章 ID（pull list 输出；draft 通道为 media_id，publish 通道为 article_id）
        article_id: String,
        /// 输出 Markdown 文件，缺省 <article_id>.md
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 平台，目前只支持 wechat
        #[arg(long, default_value = "wechat")]
        platform: String,
        /// 账号名覆盖（缺省：环境变量 > `config set account`）
        #[arg(long)]
        account: Option<String>,
        /// provider 地址覆盖（缺省：环境变量 > `config set provider` > 本机 8787）
        #[arg(long)]
        provider: Option<String>,
        /// 通道：draft＝草稿箱（缺省），publish＝已发布
        #[arg(long)]
        source: Option<String>,
    },
}

#[derive(Subcommand)]
enum NoteCmd {
    /// 导出格式 → 小红书图集 PNG（1080×1440，3:4；封面 + 正文分页，纯本地生成）
    Export {
        /// Markdown 文件或草稿名
        markdown: String,
        /// 输出目录，缺省：正文旁 `<名称>-rednote/`（草稿在草稿区旁）
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// 设置项：account（provider 侧账号名）/ provider（地址）
    Set { key: String, value: String },
    /// 查看当前配置与生效值
    Show,
}

#[derive(Subcommand)]
enum DraftCmd {
    /// 列出草稿（名称、状态、标题）
    List,
    /// 新建草稿（生成正文骨架，可在 CLI 或 studio 中继续写）
    New { name: String },
    /// 查看草稿：正文路径与 meta 状态
    Show { name: String },
    /// 删除草稿（正文与 meta 一并清理）
    Delete { name: String },
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
        Cmd::Article { cmd } => match cmd {
            ArticleCmd::Render {
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
            ArticleCmd::Push {
                markdown,
                platform,
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
                require_platform(&platform)?;
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
            ArticleCmd::Pull { cmd } => match cmd {
                PullCmd::List {
                    platform,
                    account,
                    provider,
                    source,
                    count,
                    offset,
                } => {
                    require_platform(&platform)?;
                    let base = crate::config::provider_base(provider.as_deref());
                    let account = crate::config::resolve_account(account.as_deref())?;
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
                        "共 {total} 篇（本页 {}，offset {offset}，账号 {account}，通道 {source}）",
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
                PullCmd::Get {
                    article_id,
                    output,
                    platform,
                    account,
                    provider,
                    source,
                } => {
                    require_platform(&platform)?;
                    let base = crate::config::provider_base(provider.as_deref());
                    let account = crate::config::resolve_account(account.as_deref())?;
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
                        println!("封面 media_id：{t}（article push --thumb 直接用）");
                    }
                    if let Some(u) = v["url"].as_str().filter(|s| !s.is_empty()) {
                        println!("原文：{u}");
                    }
                    println!(
                        "改完后 `article push` 可直接用此文件（标题走 YAML front matter）。\n注意：文中图片仍是原文链接，如需入库需重传。"
                    );
                    Ok(())
                }
            },
            ArticleCmd::Draft { cmd } => match cmd {
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
                    println!("在 CLI 或 studio 里继续写，push 时用草稿名「{name}」。");
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
                DraftCmd::Delete { name } => {
                    let path = drafts::markdown_path(&name);
                    if !path.exists() {
                        bail!("草稿不存在：{}", path.display());
                    }
                    std::fs::remove_file(&path)
                        .with_context(|| format!("删除草稿失败：{}", path.display()))?;
                    let meta = drafts::meta_path(&name);
                    if meta.exists() {
                        std::fs::remove_file(&meta)
                            .with_context(|| format!("删除 meta 失败：{}", meta.display()))?;
                    }
                    println!("已删除草稿「{name}」（正文与 meta 一并清理）");
                    Ok(())
                }
            },
        },
        Cmd::Note { cmd } => match cmd {
            NoteCmd::Export { markdown, output } => {
                let input = drafts::resolve_input(&markdown)?;
                let text = std::fs::read_to_string(input.path())
                    .with_context(|| format!("读取正文失败：{}", input.path().display()))?;
                let out = match output {
                    Some(o) => o,
                    None => match &input {
                        drafts::Input::Draft { name, .. } => {
                            drafts::drafts_dir().join(format!("{name}-rednote"))
                        }
                        drafts::Input::File(p) => {
                            let dir = p.parent().unwrap_or_else(|| Path::new("."));
                            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("article");
                            dir.join(format!("{stem}-rednote"))
                        }
                    },
                };
                let paths = crate::rednote::generate(&text, &out)?;
                for p in &paths {
                    println!("{}", p.display());
                }
                println!("共 {} 页，按序号上传小红书即可。", paths.len());
                Ok(())
            }
        },
        Cmd::Config { cmd } => match cmd {
            ConfigCmd::Set { key, value } => {
                let c = crate::config::set(&key, &value)?;
                println!(
                    "已设置 {key}={}（{}）",
                    value,
                    crate::config::path_display()
                );
                let _ = c;
                Ok(())
            }
            ConfigCmd::Show => {
                let c = crate::config::load();
                println!("账号：{}", c.account.as_deref().unwrap_or("（未设置）"));
                println!(
                    "provider：{}",
                    c.provider.as_deref().unwrap_or("（未设置，缺省本机 8787）")
                );
                println!(
                    "环境变量可覆盖：QTCLOUD_MEDIA_ACCOUNT / QTCLOUD_MEDIA_PROVIDER；配置文件 {}",
                    crate::config::path_display()
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
    }
}

/// 平台参数校验：目前只接 wechat；轴留好，未来平台在这里扩。
fn require_platform(platform: &str) -> Result<()> {
    if platform != "wechat" {
        bail!("暂只支持平台 wechat（收到：{platform}）");
    }
    Ok(())
}

/// 远程模式：把推送请求交给 provider（账号托管、发布记录都在服务端；
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

    let account = crate::config::resolve_account(account)?;
    let base = crate::config::provider_base(provider);
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
