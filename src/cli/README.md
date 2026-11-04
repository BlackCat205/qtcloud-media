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
qtcloud-media article render article.md --theme themes/default.toml -o article.html

# 排版 + 推入公众号草稿箱（不群发）
export QTCLOUD_MEDIA_APP_ID=wx...
export QTCLOUD_MEDIA_APP_SECRET=...
qtcloud-media article push article.md --theme themes/default.toml \
  --title "文章标题" --thumb ./cover.jpg --digest "摘要"
```

`article push` 会自动处理三件事：

1. Markdown 渲染成带主题样式的 HTML 片段
2. 正文里的本地图片逐张上传（`media/uploadimg`），替换成微信 URL
3. 封面上传为永久素材（`material/add_material`），新增草稿（`draft/add`）

文章停在**草稿箱**，不会自动群发；到公众号后台确认后手动群发。

### 草稿接力（CLI ↔ studio）

草稿是 XDG 状态数据，与主题（资产）分开存放：`$XDG_STATE_HOME/qtcloud-media/drafts/`（回退 `~/.local/state/...`）。一个草稿两个文件：`<name>.md`（正文）+ `<name>.meta.json`（title/digest/thumb/theme/media_id，由 `article push` 维护）。

```bash
qtcloud-media article draft new 我的文章          # 新建（studio 可直接打开继续写）
qtcloud-media article draft list                 # 名称、是否已推草稿箱、标题
qtcloud-media article draft show 我的文章         # 正文路径与 meta 状态
qtcloud-media article push 我的文章 --title ... --thumb ...
# 首次推入草稿箱；再次推送同名草稿走 draft/update 更新同一篇，不新建
```

`article render` 的位置参数同样接受草稿名。正式要进 brochure/library 的定稿仍从草稿移入仓库，git 是正本。

### 远程模式（接 provider）

local（缺省）：本机直连微信。remote：草稿推送交给 provider（账号托管、推送记录在服务端；草稿与主题仍是你本机的 XDG 正本）。

```bash
# 先把账号交给 provider（一次性）
curl -X PUT http://127.0.0.1:8787/accounts/主号 -H 'content-type: application/json' \
  -d '{"app_id":"...","app_secret":"..."}'

qtcloud-media article push 我的文章 --remote --account 主号 --title ... --thumb ...
# 封面是本地路径时自动转 base64（provider 按魔数嗅探类型）；
# provider 地址缺省 http://127.0.0.1:8787，可通过 --provider、QTCLOUD_MEDIA_PROVIDER 或 config set provider 设置
```

### 技能管理

技能文件编译期内嵌进二进制，随 `cargo install` 带到任何机器：

```bash
# 列出内嵌技能
qtcloud-media skills list

# 安装到本机 agent 技能目录（缺省 ~/.agents/skills）
qtcloud-media skills install              # 全部
qtcloud-media skills install qtcloud-media-wechat --dir /path/to/skills
```

## 凭证

| 环境变量 | 说明 |
|---|---|
| `QTCLOUD_MEDIA_APP_ID` | 公众号 AppID（须为已认证、有 API 权限的账号） |
| `QTCLOUD_MEDIA_APP_SECRET` | 公众号 AppSecret |

注意：公众号接口要求请求方 IP 在公众号后台的「IP 白名单」内，部署前先把服务器出口 IP 配进去。

## 主题

主题是一个扁平 TOML 文件，承载一整套排版参数（字号、行高、间距、标题、引用、代码、配色）。换主题不碰正文。`--theme` 既可以传文件路径，也可以传名称；传名称时按 XDG 顺序查找，CLI 与 studio 读同一份：

| XDG 变量 | 路径 | 内容 |
|---|---|---|
| `XDG_DATA_HOME` | `~/.local/share/qtcloud-media/themes/<name>.toml` | 主题资产（首选） |
| `XDG_CONFIG_HOME` | `~/.config/qtcloud-media/` | `secret.env`、`themes/`（次选） |
| `XDG_CACHE_HOME` | `~/.cache/qtcloud-media/` | 预留缓存位 |

环境变量不存在或为相对路径时回退家目录默认位置（与 studio 的 `paths_io.dart` 同一规则）。

## 技能

`skills/qtcloud-media-wechat`：AI 助手用本 CLI 推公众号草稿箱的技能文件（SKILL.md）。
