# qtcloud_media_studio

量潮媒体云工作台（Flutter，Web + Linux 桌面）。

单页工作台：左侧写 Markdown，右侧实时预览公众号排版，点「复制到公众号」把带样式的富文本写入剪贴板，到公众号编辑器直接粘贴即可——对应 CLI `publish` 链路里的「排版」一半，不联网、不碰凭证。

## 平台

| 平台 | 预览 | 富文本复制 |
|------|------|-----------|
| Web | iframe（`preview_web.dart`） | ClipboardItem 写 text/html（`web/index.html` 的 `copyRichText`） |
| Linux 桌面 | Flutter Widget 按块模型渲染（`preview_desktop.dart`） | GTK 剪贴板双 target：UTF8_STRING + text/html（`linux/runner/my_application.cc`，通道 `qtcloud_media/clipboard`） |

两平台共用 `lib/md.dart` 的同一套解析与样式参数（与 CLI render 同规则：无 `<style>`、无 class、开头 H1 不进正文）。

## 运行 / 安装

```bash
# Web
flutter pub get && flutter run -d web-server --web-port 8080

# Linux 桌面
flutter build linux
cp -r build/linux/x64/release/bundle ~/.local/opt/qtcloud-media-studio
ln -sf ~/.local/opt/qtcloud-media-studio/qtcloud_media_studio ~/.local/bin/qtcloud-media-studio
# 应用菜单入口：~/.local/share/applications/qtcloud-media-studio.desktop
```

## 结构

| 文件 | 职责 |
|------|------|
| `lib/md.dart` | Markdown → 块模型 → 公众号 HTML（逐元素内联样式） |
| `lib/preview_*.dart` / `lib/copy_*.dart` | 平台条件导入：预览与富文本复制的两套实现 |
| `lib/main.dart` | 单页工作台：编辑 + 预览 + 复制 |
| `lib/drafts_io.dart` | 草稿读写：与 CLI 共用 `~/.local/state/qtcloud-media/drafts/`（打开/保存） |
| `lib/theme.dart` | 主题参数（与 CLI theme.rs 同字段的扁平 TOML） |
| `linux/runner/my_application.cc` | GTK 富文本剪贴板方法通道 |

## 与 CLI 的关系

排版规则同源；主题参数共用同一份 XDG 文件：启动时按 CLI 同一查找顺序读 `~/.local/share/qtcloud-media/themes/default.toml`（次选 `~/.config/...`），找不到回退内置默认（Web 端直接内置）。路径规则见 CLI 仓 `src/cli/src/paths.rs` 与本仓 `lib/paths_io.dart`——一字不差。发布（草稿箱直出）在 CLI，复制粘贴在工作台。
