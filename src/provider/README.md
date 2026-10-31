# qtcloud-media-provider

量潮媒体云 provider：**render / publish / 账号 / 主题**四动词薄服务。商业化架构里的收费层——CLI 与 studio 永远免费、本地资产永远在 XDG；provider 做多账号托管、批量发布、发布记录审计。

## 定位

- **本地正本**：草稿与 meta 在 `~/.local/state/qtcloud-media/drafts/`（与 CLI 接力共用）、主题在 `~/.local/share/qtcloud-media/themes/`（与 CLI 共用）；provider 挂了用户资产不受影响
- **复用单源**：直接依赖 `qtcloud-media-cli`，排版引擎与微信客户端与 CLI 同一份实现（家族 provider 为 Go；此处不重写第三份排版实现，单源优先于语言惯例）
- **默认本机**：监听 `127.0.0.1:8787`，无鉴权层（自用阶段）；对外暴露前必须加鉴权——那是部署到公网时的事

## 接口

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/health` | 状态与 XDG 路径 |
| POST | `/render` | `{markdown, theme?}` → `{html}` |
| POST | `/publish` | `{account, markdown, title, digest?, author?, source_url?, thumb_media_id?/thumb_data_b64?/thumb_url?, theme?, draft?}` → `{media_id, action}`；带 `draft` 走接力（带 media_id 更新同一篇） |
| GET | `/accounts` | 账号名列表（不回显密钥） |
| PUT | `/accounts/{name}` | `{app_id, app_secret}`，落盘 chmod 600 |
| GET/PUT | `/themes/{name}` | 主题 TOML，与 CLI 共用目录 |
| GET | `/records` | 发布记录（`~/.local/state/qtcloud-media/provider/records.jsonl`） |

## 运行

```bash
cargo run -p qtcloud-media-provider   # 或 target/debug/qtcloud-media-provider
# 监听地址用 QTCP_PROVIDER_LISTEN 覆盖，默认 127.0.0.1:8787
```

## 现状（自用阶段）

- 已用真实公众号端到端首验：推入草稿箱 → 二次 publish 走 draft/update 更新同一篇 → 发布记录落盘
- 未做（商业化部署时补）：鉴权、多租户、部署形态（FC/容器）、收款
