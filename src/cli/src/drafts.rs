//! 草稿：CLI 与 studio 共用的接力工作区。
//!
//! 位置：`$XDG_STATE_HOME/qtcloud-media/drafts/`（回退 `~/.local/state/...`）。
//! 一个草稿两个文件：`<name>.md`（正文，纯 Markdown）+
//! `<name>.meta.json`（title/digest/thumb/theme/media_id）。
//! meta 里的 `media_id` 是接力关键：二次 publish 走微信 draft/update
//! 更新同一篇草稿，而不是每次新建。

use crate::paths;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DraftMeta {
    pub title: String,
    pub digest: String,
    pub thumb: String,
    pub theme: String,
    /// 上次 publish 的草稿 media_id；空串表示尚未推过草稿箱
    pub media_id: String,
}

/// 位置参数解析结果：普通文件，或草稿名。
#[derive(Debug, Clone)]
pub enum Input {
    File(PathBuf),
    Draft { name: String, path: PathBuf },
}

impl Input {
    pub fn path(&self) -> &Path {
        match self {
            Input::File(p) => p,
            Input::Draft { path, .. } => path,
        }
    }
}

pub fn drafts_dir() -> PathBuf {
    paths::state_dir().join("drafts")
}

pub fn markdown_path(name: &str) -> PathBuf {
    drafts_dir().join(format!("{name}.md"))
}

pub fn meta_path(name: &str) -> PathBuf {
    drafts_dir().join(format!("{name}.meta.json"))
}

/// 位置参数：先当文件路径，再当草稿名。
pub fn resolve_input(spec: &str) -> Result<Input> {
    resolve_input_in(spec, &drafts_dir())
}

pub fn resolve_input_in(spec: &str, drafts: &Path) -> Result<Input> {
    let direct = Path::new(spec);
    if direct.is_file() {
        return Ok(Input::File(direct.to_path_buf()));
    }
    let draft_md = drafts.join(format!("{spec}.md"));
    if draft_md.is_file() {
        return Ok(Input::Draft {
            name: spec.to_string(),
            path: draft_md,
        });
    }
    bail!(
        "找不到「{spec}」：既不是文件，也不在 {} 下的草稿",
        drafts.display()
    )
}

pub fn list() -> Result<Vec<String>> {
    list_in(&drafts_dir())
}

pub fn list_in(drafts: &Path) -> Result<Vec<String>> {
    if !drafts.is_dir() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = std::fs::read_dir(drafts)
        .with_context(|| format!("读取草稿目录失败：{}", drafts.display()))?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .filter(|f| f.ends_with(".md"))
        .map(|f| f.trim_end_matches(".md").to_string())
        .collect();
    names.sort();
    Ok(names)
}

pub fn load_meta(name: &str) -> DraftMeta {
    load_meta_in(&meta_path(name))
}

pub fn load_meta_in(path: &Path) -> DraftMeta {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_meta(name: &str, meta: &DraftMeta) -> Result<()> {
    save_meta_in(&meta_path(name), meta)
}

pub fn save_meta_in(path: &Path, meta: &DraftMeta) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("创建草稿目录失败：{}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(meta).context("序列化草稿 meta 失败")?;
    std::fs::write(path, json).with_context(|| format!("写入失败：{}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("qtm-drafts-{tag}-{}", std::process::id()))
    }

    #[test]
    fn resolves_file_then_draft_name() {
        let d = tmp("resolve");
        std::fs::create_dir_all(&d).unwrap();
        let file = d.join("article.md");
        std::fs::write(&file, "x").unwrap();
        std::fs::write(d.join("mine.md"), "y").unwrap();

        assert!(matches!(
            resolve_input_in(file.to_str().unwrap(), &d).unwrap(),
            Input::File(_)
        ));
        assert!(matches!(
            resolve_input_in("mine", &d).unwrap(),
            Input::Draft { .. }
        ));
        assert!(resolve_input_in("missing", &d).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn meta_roundtrip() {
        let d = tmp("meta");
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("x.meta.json");
        let meta = DraftMeta {
            title: "标题".into(),
            media_id: "MID".into(),
            ..Default::default()
        };
        save_meta_in(&p, &meta).unwrap();
        let loaded = load_meta_in(&p);
        assert_eq!(loaded.title, "标题");
        assert_eq!(loaded.media_id, "MID");
        let _ = std::fs::remove_dir_all(&d);
    }
}
