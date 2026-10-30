# Changelog

## [0.0.1] - 2026-10-30

### 新增

- 初始化仓库：Rust CLI 骨架（crate qtcloud-media-cli，bin qtcloud-media）
- `render`：Markdown → 公众号草稿 HTML，主题（TOML）驱动整套排版，不联网
- `publish`：排版 + 本地图片上传替换 + 封面永久素材 + 草稿箱直出（draft/add，不群发）
- 实测修复：上传 multipart 显式声明图片 MIME；建稿遇微信侧瞬时 40007 带退避重试（2s/4s/8s）
- `themes/default.toml`：默认主题
