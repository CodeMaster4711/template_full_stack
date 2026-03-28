use super::AppState;
use axum::body::Body;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use axum::http::{HeaderValue, Request, Response};
use axum::Router;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info_span, Span};

pub mod organizations;
pub mod users;

pub fn create_router() -> Router<AppState> {
    let rate_limit_per_second: u64 = std::env::var("RATE_LIMIT_PER_SECOND")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    let burst_size: u32 = std::env::var("RATE_LIMIT_BURST")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);

    let governor_config = GovernorConfigBuilder::default()
        .per_second(rate_limit_per_second)
        .burst_size(burst_size)
        .finish()
        .expect("invalid rate limit configuration");

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:1420".to_string());

    tracing::info!("CORS configured for frontend URL: {}", frontend_url);

    let allowed_origins = vec![
        "http://localhost:1420",
        "http://localhost:8000",
        "http://127.0.0.1:1420",
        "http://127.0.0.1:8000",
        &frontend_url,
    ];

    let cors = CorsLayer::new()
        .allow_origin(
            allowed_origins
                .into_iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true);

    let api_router = Router::new()
        .merge(users::users_routes())
        .merge(organizations::routes());

    Router::new()
        .nest("/api", api_router)
        .layer(GovernorLayer::new(governor_config))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|_request: &Request<Body>, _span: &Span| {
                    tracing::info!("started processing request")
                })
                .on_response(
                    |_response: &Response<Body>, latency: std::time::Duration, _span: &Span| {
                        tracing::info!(
                            latency_ms = latency.as_millis(),
                            "finished processing request"
                        )
                    },
                ),
        )
        .layer(cors)
}
