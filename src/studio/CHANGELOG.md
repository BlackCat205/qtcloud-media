# Changelog

## [Unreleased]

### 变更

- 信息架构调整：新增左侧导航栏（文章 / 图文 / 视频 / 设置）；现单页编辑器归入「文章」；远程设置弹窗移入「设置」页（服务端连接：provider 地址 + 账号）；「图文」「视频」为预留分区占位页
- 布局调整：「设置」入口移到导航栏底部；文章页改为「草稿清单 | 编辑器 | 预览」三栏，AppBar 的打开草稿/保存草稿按钮撤掉（草稿操作走清单：点项打开、＋存为新草稿、保存写回当前草稿）

## [0.1.3] - 2026-10-31

### 新增

- 远程模式：`lib/remote_io.dart` 接 provider——云朵图标配置 provider 地址与账号，配好后出现「推到公众号」一键推送（local 模式不变：渲染+剪贴板）

## [0.1.2] - 2026-10-31

### 新增

- 发布部署（家族惯例，参考 qtcloud-flutter-studio 模板）：`deploy-studio.yml`（分支/PR 门禁，`studio/*` tag 构建部署到 OSS）+ `manifests/terraform/`（studio-bucket + CDN/私有回源/强制 HTTPS + DNS，域名 studio.media.cloud.quanttide.com）；Windows 打包随 `release-studio.yml`（首验随下次发版）

## [0.1.1] - 2026-10-31

### 新增

- 草稿接力：`lib/drafts_io.dart` 与 CLI drafts.rs 共用 `~/.local/state/qtcloud-media/drafts/`；AppBar 新增打开草稿与保存（meta 由 CLI publish 维护，studio 只读正文）
- 主题参数化：样式统一由 `lib/theme.dart`（与 CLI theme.rs 同字段的扁平 TOML）驱动，Web/桌面预览与复制同步生效
- XDG 共享：`lib/paths_io.dart` 与 CLI `paths.rs` 同一规则；启动时读 `~/.local/share/qtcloud-media/themes/default.toml`（次选 config），找不到回退内置默认

## [0.1.0] - 2026-10-31

### 新增

- 单页工作台 MVP（Web）：Markdown 编辑 + 公众号排版实时预览（iframe）+ 富文本复制到剪贴板（text/html，粘贴进公众号编辑器保留排版）；`lib/md.dart` 内联样式转换与 CLI render 同规则
- Linux 桌面版：块模型重构（解析与 HTML/Widget 双渲染共用），GTK 富文本剪贴板方法通道（UTF8_STRING + text/html），本机安装（~/.local/opt + .desktop）
- 单元测试：md 转换（H1 跳过、全内联样式、转义、正文不变）

## [0.0.1] - 2026-10-30

### 新增

- 初始化仓库：Flutter 骨架（qtcloud_media_studio，com.quanttide）
