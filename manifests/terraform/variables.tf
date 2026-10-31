variable "region" {
  description = "阿里云地域"
  type        = string
  default     = "cn-hangzhou"
}

variable "project" {
  description = "项目名（资源命名前缀）"
  type        = string
  default     = "qtcloud-media"
}

variable "environment" {
  description = "环境：dev / prod"
  type        = string
  default     = "prod"
}

variable "oss_bucket_name" {
  description = "Studio 静态站点桶名（OSS 全局唯一；可覆盖，默认 {repo}-studio）"
  type        = string
  default     = "qtcloud-media-studio"
}
