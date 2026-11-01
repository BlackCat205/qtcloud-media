---
name: qtcloud-media-wechat
description: 用 qtcloud-media CLI 把 Markdown 文章排版并推进微信公众号草稿箱。当用户要求发公众号、推草稿、排版公众号文章，或说「把这篇 md 发到公众号」时使用。不负责群发。
---

# qtcloud-media-wechat

微信公众号草稿箱直出：Markdown → 主题排版 → 草稿箱。发布（群发）动作始终留给人。

## 前置条件

三项缺一不可，缺哪项补哪项：

1. 已安装：`cargo install qtcloud-media-cli`
2. 环境变量 `QTCLOUD_MEDIA_APP_ID` / `QTCLOUD_MEDIA_APP_SECRET`。密钥不落仓库，用时从机密库注入（如 `source ~/.config/qtcloud-media/secret.env`）
3. 公众平台后台「IP 白名单」包含本机出口 IP。报错 `40164 invalid ip` 时，先用 `curl -4 api.ipify.org` 查出口 IP，报给用户去后台添加

## 工作流

1. **确认稿件**：正文遵守公众号稿件风格——标题不夹命令名、正文不用 H1（render 会自动跳过开头 H1）、代码块前用一句自然语句说明、正文不放 GitHub 等外链（公众号里点不开）
2. **本地预览**：`qtcloud-media render article.md --theme themes/default.toml -o article.html`，检查结构无误再往下走
3. **推草稿箱**：`qtcloud-media article push article.md --title "标题" --digest "一句摘要" --thumb 封面图`。本地图片会自动上传替换，封面自动入库
4. **报告**：把 media_id 报给用户，明确说明「文章停在草稿箱，不会自动群发，需到公众号后台确认排版、必要时换封面，再手动群发」

## 注意

- **不是群发**：本技能只到草稿箱为止，群发由人在后台完成
- 封面若用占位图，报告时提醒用户替换
- 建稿遇微信侧瞬时 `40007` 已由 CLI 自动退避重试，无需人工干预；重试三次仍失败才报错
- 草稿名二次 article push 自动走 `draft/update` 更新同一篇（meta 里的 media_id）；微信接口不对称：update 的 `articles` 是单对象、add 才是数组，传错报 47001，CLI 已处理
- 标题只进草稿 title 字段，正文不重复标题；改标题改 `--title` 参数即可
- 定时发布、摘要微调在公众号后台完成，CLI 不做
