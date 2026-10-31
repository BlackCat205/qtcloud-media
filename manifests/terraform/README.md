# qtcloud-media 部署（IaC）

Studio Web 的基础设施：OSS 静态站点桶 + 阿里云 CDN（家族惯例，参考 qtcloud-secret/manifests/terraform）。

## 部署链路

```
studio/* tag → Actions（deploy-studio.yml，flutter build web + ossutil cp）
  → OSS 桶 qtcloud-media-studio（静态网站模式，私有）
  → CDN studio.media.cloud.quanttide.com（私有回源鉴权）
  → 用户浏览器
```

## 本 IaC 范围

- OSS 静态站点桶（私有 + 静态网站托管，`studio-bucket.tf`）
- CDN 域名 + 私有回源鉴权 + 根路径改写 + 强制 HTTPS + 账号级 RAM 授权（`cdn.tf`）
- DNS：CNAME `studio.media.cloud.quanttide.com` → CDN（云解析，`cdn.tf`）
- 不含：证书内容（acme.sh 单域名管理，见 cdn.tf 头注释）、发布流水线（.github/workflows/）

## 首次上线步骤

1. `terraform init -backend-config="bucket=<state桶>" -backend-config="key=qtcloud-media/terraform.tfstate" -backend-config="region=cn-hangzhou"`
2. `terraform apply`（建桶 + CDN 域名 + DNS）
3. CDN 控制台为域名开启「阿里云OSS私有Bucket回源」（无 OpenAPI，手动；见 cdn.tf 头注释）
4. acme.sh 签发 `studio.media.cloud.quanttide.com` 单域名证书并配置 reloadcmd 重配 CDN（否则仅 HTTP）
5. 确认 GitHub org secret `ALIYUN_ACCESS_KEY_ID` / `ALIYUN_ACCESS_KEY_SECRET` 已配（发布流水线用）
6. 推 `studio/*` tag 首发，验证域名可访问
