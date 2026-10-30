//! 主题：一整套可用排版。主题文件（TOML）承载全部排版参数，换主题不碰正文。

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub name: String,
    pub font_family: String,
    pub code_font_family: String,
    /// 正文字号（px）
    pub font_size: u32,
    pub line_height: f32,
    pub text_color: String,
    pub paragraph_spacing: u32,
    pub heading_color: String,
    pub heading_weight: u32,
    pub h2_size: u32,
    pub h3_size: u32,
    pub h4_size: u32,
    pub heading_top_spacing: u32,
    pub heading_bottom_spacing: u32,
    pub quote_border_color: String,
    pub quote_bg: String,
    pub quote_color: String,
    pub code_bg: String,
    pub code_color: String,
    pub link_color: String,
    pub strong_color: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".into(),
            font_family: "-apple-system,BlinkMacSystemFont,'Helvetica Neue','PingFang SC','Microsoft YaHei',sans-serif".into(),
            code_font_family: "SFMono-Regular,Menlo,Consolas,'Courier New',monospace".into(),
            font_size: 16,
            line_height: 1.8,
            text_color: "#3f3f3f".into(),
            paragraph_spacing: 16,
            heading_color: "#1f2937".into(),
            heading_weight: 600,
            h2_size: 21,
            h3_size: 18,
            h4_size: 17,
            heading_top_spacing: 28,
            heading_bottom_spacing: 14,
            quote_border_color: "#07c160".into(),
            quote_bg: "#f6f8f7".into(),
            quote_color: "#6a737d".into(),
            code_bg: "#f6f8f7".into(),
            code_color: "#c7254e".into(),
            link_color: "#576b95".into(),
            strong_color: "#1f2937".into(),
        }
    }
}

impl Theme {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("读取主题文件失败：{}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("解析主题文件失败：{}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_is_complete() {
        let t = Theme::default();
        assert_eq!(t.name, "default");
        assert_eq!(t.font_size, 16);
        assert!(t.heading_color.starts_with('#'));
    }
}
