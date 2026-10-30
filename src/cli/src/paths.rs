//! XDG Base Directory 解析。CLI 与 studio（Dart 侧 paths_io.dart）使用
//! 同一条规则：环境变量优先且必须是绝对路径，否则回退家目录默认位置；
//! 应用目录统一叫 `qtcloud-media`。

use std::path::{Path, PathBuf};

/// 配置目录：`$XDG_CONFIG_HOME/qtcloud-media` → `~/.config/qtcloud-media`
pub fn config_dir() -> PathBuf {
    base_dir("XDG_CONFIG_HOME", ".config").join("qtcloud-media")
}

/// 数据目录：`$XDG_DATA_HOME/qtcloud-media` → `~/.local/share/qtcloud-media`
pub fn data_dir() -> PathBuf {
    base_dir("XDG_DATA_HOME", ".local/share").join("qtcloud-media")
}

/// 状态目录：`$XDG_STATE_HOME/qtcloud-media` → `~/.local/state/qtcloud-media`
pub fn state_dir() -> PathBuf {
    base_dir("XDG_STATE_HOME", ".local/state").join("qtcloud-media")
}

/// 缓存目录：`$XDG_CACHE_HOME/qtcloud-media` → `~/.cache/qtcloud-media`
pub fn cache_dir() -> PathBuf {
    base_dir("XDG_CACHE_HOME", ".cache").join("qtcloud-media")
}

fn base_dir(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute()) // XDG 规范：相对路径视为未设置
        .unwrap_or_else(|| home_dir().join(fallback))
}

fn home_dir() -> PathBuf {
    #[allow(deprecated)] // std::env::home_dir 已在新版本解除弃用
    std::env::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// 主题查找：先当文件路径，再按名称查 `<data>/themes/<name>.toml`、
/// `<config>/themes/<name>.toml`。返回找到的文件路径。
pub fn resolve_theme(spec: &str) -> Option<PathBuf> {
    resolve_theme_in(spec, &data_dir(), &config_dir())
}

/// 纯函数版本，便于测试注入基准目录。
pub fn resolve_theme_in(spec: &str, data: &Path, config: &Path) -> Option<PathBuf> {
    let direct = Path::new(spec);
    if direct.is_file() {
        return Some(direct.to_path_buf());
    }
    for base in [data, config] {
        let candidate = base.join("themes").join(format!("{spec}.toml"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_name_from_data_then_config() {
        let tmp = std::env::temp_dir().join(format!("qtm-xdg-test-{}", std::process::id()));
        let data = tmp.join("data");
        let config = tmp.join("config");
        std::fs::create_dir_all(data.join("themes")).unwrap();
        std::fs::create_dir_all(config.join("themes")).unwrap();
        std::fs::write(data.join("themes/a.toml"), "name = \"a\"").unwrap();
        std::fs::write(config.join("themes/b.toml"), "name = \"b\"").unwrap();

        assert_eq!(
            resolve_theme_in("a", &data, &config),
            Some(data.join("themes/a.toml"))
        );
        assert_eq!(
            resolve_theme_in("b", &data, &config),
            Some(config.join("themes/b.toml"))
        );
        assert_eq!(resolve_theme_in("missing", &data, &config), None);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
