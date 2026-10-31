//! qtcloud-media provider 入口。
//!
//! 默认只监听 127.0.0.1（本机自用，无鉴权层）；对外暴露前必须加鉴权，
//! 这是商业化部署时的事。监听地址用环境变量 QTCP_PROVIDER_LISTEN 覆盖。

use qtcloud_media_provider::{AppState, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listen =
        std::env::var("QTCP_PROVIDER_LISTEN").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    let state = AppState::from_xdg();
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    println!("qtcloud-media-provider 监听 http://{listen}");
    axum::serve(listener, app).await?;
    Ok(())
}
