use actix_web::{web, HttpRequest, HttpResponse};
use futures_util::StreamExt as _;

/// WebSocket Live Query Streaming Actor (Milestone 1.8 & 4.3).
pub async fn live_ws_handler(
    req: HttpRequest,
    body: web::Payload,
) -> actix_web::Result<HttpResponse> {
    let (res, mut session, mut stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.next().await {
            if let actix_ws::Message::Text(text) = msg {
                // In production this relays SurrealDB LIVE SELECT events
                let ack_frame = format!("LIVE_EVENT:{}", text);
                let _ = session.text(ack_frame).await;
            }
        }
    });

    Ok(res)
}

/// Generates a SurrealQL LIVE SELECT registration query.
#[must_use]
pub fn live_query(tenant_ns: &str, table: &str, id: &str) -> String {
    format!(
        "LIVE SELECT * FROM {}:{} WHERE $tenant = '{}';",
        table, id, tenant_ns
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_query_format() {
        let q = live_query("tenant_acme", "sales_invoice", "INV-001");
        assert!(q.starts_with("LIVE SELECT * FROM sales_invoice:INV-001"));
    }
}
