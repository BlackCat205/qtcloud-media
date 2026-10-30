# qtcloud_media_studio

量潮媒体云工作台（Flutter Web）。

当前为单页 MVP：左侧写 Markdown，右侧实时预览公众号排版，点「复制到公众号」把带样式的富文本写入剪贴板，到公众号编辑器直接粘贴即可——对应 CLI `publish` 链路里的「排版」一半，不联网、不碰凭证。

## 运行

```bash
flutter pub get
flutter run -d web-server --web-port 8080
```

## 结构

| 文件 | 职责 |
|------|------|
| `lib/md.dart` | Markdown → 公众号 HTML（逐元素内联样式，与 CLI render 同规则：无 `<style>`、无 class、开头 H1 不进正文） |
| `lib/main.dart` | 单页工作台：编辑 + iframe 预览 + 富文本复制 |
| `web/index.html` | `copyRichText`：ClipboardItem 同时写 text/html 与 text/plain |

## 与 CLI 的关系

排版规则同源（同一套主题参数与内联策略）。发布（草稿箱直出）在 CLI，复制粘贴在工作台。
