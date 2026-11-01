//! CLI 连接设置：provider 与账号，配一次各命令共用。
//!
//! 优先级：命令行 flag > 环境变量 > 本文件（`~/.config/qtcloud-media/cli.json`）。
//! studio 的连接设置在 studio.json，两侧同目录各自维护。

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct CliConfig {
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
}

fn path() -> std::path::PathBuf {
    crate::paths::config_dir().join("cli.json")
}

/// 配置文件路径（展示用）。
pub fn path_display() -> String {
    path().display().to_string()
}

pub fn load() -> CliConfig {
    std::fs::read_to_string(path())
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

/// `config set account|provider <值>`。未知项报错。
pub fn set(key: &str, value: &str) -> Result<CliConfig> {
    let mut c = load();
    match key {
        "account" => c.account = Some(value.to_string()),
        "provider" => c.provider = Some(value.to_string()),
        _ => bail!("未知配置项「{key}」：目前支持 account、provider"),
    }
    let p = path();
    let parent = p.parent().context("配置路径缺少父目录")?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("创建配置目录失败：{}", parent.display()))?;
    std::fs::write(&p, serde_json::to_string_pretty(&c)?)
        .with_context(|| format!("写入配置失败：{}", p.display()))?;
    Ok(c)
}

/// provider 地址：flag > 环境变量 QTCLOUD_MEDIA_PROVIDER > 配置文件 > 缺省本机。
pub fn provider_base(flag: Option<&str>) -> String {
    flag.map(str::to_owned)
        .or_else(|| std::env::var("QTCLOUD_MEDIA_PROVIDER").ok())
        .or_else(|| load().provider)
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string())
}

/// 账号：flag > 环境变量 QTCLOUD_MEDIA_ACCOUNT > 配置文件；都没有则报错。
pub fn resolve_account(flag: Option<&str>) -> Result<String> {
    flag.map(str::to_owned)
        .or_else(|| std::env::var("QTCLOUD_MEDIA_ACCOUNT").ok())
        .or_else(|| load().account)
        .context("缺少账号：--account <名>，或先 `qtcloud-media config set account <名>`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_rejects_unknown_key() {
        assert!(set("nope", "x").is_err());
    }
}
