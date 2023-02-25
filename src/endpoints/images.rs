use worker::*;

use crate::services;
use crate::models::image_overlay_req::ImageOverlayReq;

/// Endpoint: POST /images/overlays
pub async fn ep_add_image_overlay(mut req: Request) -> Result<Response> {
    let body:ImageOverlayReq = req.json().await?;

    let image_bytes = services::image_processing::process_image_overlay(body).await;

    match image_bytes {
        Ok(image) => return Response::from_bytes(image),
        Err(e) => return Response::error(e.to_string(), 500)
    };
}
