# Changelog

## [0.1.4] - 2026-10-31

### 新增

- 远程模式：`publish --remote --account <名>` 走 provider（地址 `--provider` / `QTCLOUD_MEDIA_PROVIDER` / 缺省本机 8787）；本地封面自动转 base64
- 为 provider 开放库接口：`render_file` / `inline_local_images` 公开，新增 `upload_thumb_bytes`（封面来自 base64/URL、无本地文件时用）

## [0.1.3] - 2026-10-31

### 变更

- 修复：`update_draft` 的 `articles` 改传单对象——微信 draft/update 与 draft/add 的参数形态不对称（数组报 47001），已用真实公众号首验通过

## [0.1.2] - 2026-10-31

### 变更

- 草稿接力：新增 `drafts.rs`（`$XDG_STATE_HOME/.../drafts/`，`<name>.md` + `<name>.meta.json`）与 `draft` 命令组（list/new/show）；位置参数支持草稿名；publish 对带 media_id 的草稿走 `draft/update` 更新同一篇；微信客户端新增 `update_draft`
- XDG 共享：新增 `paths.rs`（config/data/cache 三目录，环境变量优先、相对路径视为未设置）；`--theme` 支持按名称查 `<data|config>/themes/<name>.toml`，与 studio 共用同一份主题文件

## [0.1.1] - 2026-10-31

### 变更

- 新增 `skills` 命令组（list / install）：技能文件经 agent-skills-rs 编译期内嵌，`skills install` 装到本机 agent 技能目录（缺省 ~/.agents/skills）
- 新增 `src/cli/skills/qtcloud-media-wechat`：推公众号草稿箱技能（SKILL.md）
- render：开头 H1 不再进正文（标题由草稿 title 字段承载，人工改稿实测确认）

## [0.1.0] - 2026-10-30

### 新增

- 初始化仓库：Rust CLI 骨架（crate qtcloud-media-cli，bin qtcloud-media）
- `render`：Markdown → 公众号草稿 HTML，主题（TOML）驱动整套排版，不联网
- `publish`：排版 + 本地图片上传替换 + 封面永久素材 + 草稿箱直出（draft/add，不群发）
- 实测修复：上传 multipart 显式声明图片 MIME；建稿遇微信侧瞬时 40007 带退避重试（2s/4s/8s）
- 排版重写：改为逐元素全内联样式（去 `<style>` 块与 class），适配微信草稿箱清洗；参考 WeMD（MIT）的微信适配目标，其剪贴板 DOM 路线不适用于 API 通道
- `themes/default.toml`：默认主题
