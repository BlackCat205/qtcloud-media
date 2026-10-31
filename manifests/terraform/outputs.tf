output "oss_bucket" {
  description = "Studio 静态站点桶名"
  value       = alicloud_oss_bucket.studio.bucket
}

output "cdn_domain" {
  description = "CDN 域名（studio.media.cloud.quanttide.com）"
  value       = alicloud_cdn_domain_new.studio.domain_name
}

output "cdn_cname" {
  description = "CDN 分配的 CNAME（DNS 记录的指向值）"
  value       = alicloud_cdn_domain_new.studio.cname
}
