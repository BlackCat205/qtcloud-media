locals {
  # 应用级资源命名：<app>-<env>（系统级资源由 quanttide-platform 管理）
  app_name_prefix = "${var.project}-${var.environment}"

  # Studio 静态站点桶：命名对齐站点规范 {repo}-{type}
  oss_bucket = var.oss_bucket_name
}
