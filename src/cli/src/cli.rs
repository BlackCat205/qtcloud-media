//! 命令行入口：render（只排版）、publish（排版 + 推草稿箱）、skills（技能管理）。

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
        /// Markdown 文件
        markdown: PathBuf,
        /// 主题文件（TOML），缺省用内置默认主题
        #[arg(long)]
        theme: Option<PathBuf>,
        /// 输出 HTML 文件，缺省打印到标准输出
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Markdown → 排版 → 公众号草稿箱（不群发）
    Publish {
        /// Markdown 文件
        markdown: PathBuf,
        /// 主题文件（TOML），缺省用内置默认主题
        #[arg(long)]
        theme: Option<PathBuf>,
        /// 文章标题
        #[arg(long)]
        title: String,
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
    },
    /// 技能管理：列出 / 安装内嵌的 agent 技能文件
    Skills {
        #[command(subcommand)]
        cmd: SkillsCmd,
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
            let html = render_file(&markdown, theme.as_deref())?;
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
        } => {
            let html = render_file(&markdown, theme.as_deref())?;
            let wechat = Wechat::authorize(&env_var(ENV_APP_ID)?, &env_var(ENV_APP_SECRET)?)?;
            let html = inline_local_images(&html, &markdown, &wechat)?;
            let thumb_media_id = if thumb.starts_with("http") || Path::new(&thumb).is_file() {
                wechat.upload_thumb(Path::new(&thumb))?
            } else {
                thumb
            };
            let draft_id = wechat.add_draft(Article {
                title,
                author,
                digest,
                content: html,
                content_source_url: source_url,
                thumb_media_id,
                need_open_comment: 0,
                only_fans_can_comment: 0,
            })?;
            println!("已推入公众号草稿箱，media_id={draft_id}");
            println!("注意：草稿不会自动群发，请到公众号后台确认后手动群发。");
            Ok(())
        }
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

fn embedded_skills() -> Vec<agent_skills_rs::Skill> {
    vec![
        agent_skills_rs::register_embedded_skill(WECHAT_SKILL, &[])
            .expect("内嵌技能 frontmatter 必须含 name 与 description"),
    ]
}

fn render_file(markdown: &Path, theme: Option<&Path>) -> Result<String> {
    let text = std::fs::read_to_string(markdown)
        .with_context(|| format!("读取 Markdown 失败：{}", markdown.display()))?;
    let theme = match theme {
        Some(path) => Theme::load(path)?,
        None => Theme::default(),
    };
    Ok(render::render(&text, &theme))
}

/// 把 content 里的本地图片替换为已上传的微信 URL。
fn inline_local_images(html: &str, markdown: &Path, wechat: &Wechat) -> Result<String> {
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
