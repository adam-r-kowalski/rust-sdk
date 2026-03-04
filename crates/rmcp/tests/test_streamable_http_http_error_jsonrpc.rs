#![cfg(all(
    feature = "transport-streamable-http-client",
    feature = "transport-streamable-http-client-reqwest"
))]

use std::{collections::HashMap, sync::Arc};

use axum::{http::StatusCode, routing::post};
use rmcp::{
    model::{ClientJsonRpcMessage, ClientRequest, PingRequest, RequestId, ServerJsonRpcMessage},
    transport::streamable_http_client::{StreamableHttpClient, StreamableHttpPostResponse},
};

#[tokio::test]
async fn http_400_jsonrpc_error_is_parsed_as_jsonrpc_error() -> anyhow::Result<()> {
    let router = axum::Router::new().route(
        "/mcp",
        post(|| async {
            (
                StatusCode::BAD_REQUEST,
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"Unknown prompt: example_prompt"}}"#,
            )
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });

    let uri = Arc::<str>::from(format!("http://{addr}/mcp"));
    let message = ClientJsonRpcMessage::request(
        ClientRequest::PingRequest(PingRequest::default()),
        RequestId::Number(1),
    );

    let result = reqwest::Client::new()
        .post_message(uri, message, None, None, HashMap::new())
        .await;

    match result {
        Ok(StreamableHttpPostResponse::Json(ServerJsonRpcMessage::Error(error), _)) => {
            assert_eq!(error.id, RequestId::Number(1));
            assert_eq!(
                error.error.message.as_ref(),
                "Unknown prompt: example_prompt"
            );
        }
        other => panic!("expected JSON-RPC error response, got: {other:?}"),
    }

    Ok(())
}
