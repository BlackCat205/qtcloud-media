# qtcloud-media CLI

量潮媒体云命令行工具：公众号排版与草稿箱直出。

## 安装

```bash
cargo install --path .
```

## 用法

> 渲染产物为逐元素全内联样式：不含 `<style>` 块与 class（微信草稿箱会清洗这两者）。

```bash
# 只排版：Markdown → 公众号草稿 HTML（不联网）
qtcloud-media render article.md --theme themes/default.toml -o article.html

# 排版 + 推入公众号草稿箱（不群发）
export QTCLOUD_MEDIA_APP_ID=wx...
export QTCLOUD_MEDIA_APP_SECRET=...
qtcloud-media publish article.md --theme themes/default.toml \
  --title "文章标题" --thumb ./cover.jpg --digest "摘要"
```

`publish` 会自动处理三件事：

1. Markdown 渲染成带主题样式的 HTML 片段
2. 正文里的本地图片逐张上传（`media/uploadimg`），替换成微信 URL
3. 封面上传为永久素材（`material/add_material`），新增草稿（`draft/add`）

文章停在**草稿箱**，不会自动群发；到公众号后台确认后手动群发。

## 凭证

| 环境变量 | 说明 |
|---|---|
| `QTCLOUD_MEDIA_APP_ID` | 公众号 AppID（须为已认证、有 API 权限的账号） |
| `QTCLOUD_MEDIA_APP_SECRET` | 公众号 AppSecret |

注意：公众号接口要求请求方 IP 在公众号后台的「IP 白名单」内，部署前先把服务器出口 IP 配进去。

## 主题

主题是一个 TOML 文件，承载一整套排版参数（字号、行高、间距、标题、引用、代码、配色）。换主题不碰正文，见 `themes/default.toml`。

## 技能

`skills/qtcloud-media-wechat`：AI 助手用本 CLI 推公众号草稿箱的技能文件（SKILL.md）。
