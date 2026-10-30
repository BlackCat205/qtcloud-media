# Changelog

## [Unreleased]

### 新增

- 单页工作台 MVP（Web）：Markdown 编辑 + 公众号排版实时预览（iframe）+ 富文本复制到剪贴板（text/html，粘贴进公众号编辑器保留排版）；`lib/md.dart` 内联样式转换与 CLI render 同规则
- Linux 桌面版：块模型重构（解析与 HTML/Widget 双渲染共用），GTK 富文本剪贴板方法通道（UTF8_STRING + text/html），本机安装（~/.local/opt + .desktop）
- 单元测试：md 转换（H1 跳过、全内联样式、转义、正文不变）

## [0.0.1] - 2026-10-30

### 新增

- 初始化仓库：Flutter 骨架（qtcloud_media_studio，com.quanttide）
