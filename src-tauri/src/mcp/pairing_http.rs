#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PairRequest {
    name: String,
    code: String,
}

async fn handle_pair(state: McpState, request: PairRequest) -> impl IntoResponse {
    let Some(registry) = state.pairing_registry else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Pairing registry unavailable",
        )
            .into_response();
    };
    let now = chrono::Utc::now().timestamp();
    let result = registry
        .lock()
        .map_err(|_| super::pairing::PairError::InvalidCode)
        .and_then(|mut registry| registry.pair(&request.name, &request.code, now));
    match result {
        Ok(issued) => (
            StatusCode::OK,
            Json(json!({
                "agentId": issued.id,
                "name": issued.name,
                "token": issued.token,
                "tokenType": "Bearer"
            })),
        )
            .into_response(),
        Err(_) => (StatusCode::UNAUTHORIZED, "Pairing code rejected").into_response(),
    }
}

async fn require_paired_agent(
    State(registry): State<Arc<std::sync::Mutex<super::pairing::PairingRegistry>>>,
    request: Request,
    next: Next,
) -> Response {
    if request.uri().path() == "/pair" || request.uri().path() == "/health" {
        return next.run(request).await;
    }
    let provided = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let authorized = provided.is_some_and(|token| {
        registry
            .lock()
            .map(|registry| registry.authenticate(token))
            .unwrap_or(false)
    });
    if !authorized {
        let mut response = (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        return response;
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
