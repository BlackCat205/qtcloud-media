# Changelog

## [Unreleased]

### 变更

- base64 封面按魔数字节嗅探类型（png/jpg/gif/webp），不再假定 png
- 新增 GET /articles 与 GET /articles/{id}（历史文章，query：account、source=draft|publish；draft 缺省走草稿箱，publish 走发布能力）

## [0.1.0] - 2026-10-31

### 新增

- provider MVP：四动词薄服务（render / publish / 账号 / 主题）+ 发布记录；复用 qtcloud-media-cli 排版与微信客户端（upload_thumb_bytes 新增）；草稿接力与 CLI 共用 XDG drafts/meta；默认监听 127.0.0.1:8787，账号文件 chmod 600；6 个测试（账号/主题/记录/render 端点/路径穿越拒绝）；真实公众号端到端首验通过（推入→更新同一篇）
