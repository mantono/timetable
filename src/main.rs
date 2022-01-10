mod event;

#[tokio::main]
async fn main() {}

/// - PUT /v1/schedule/{namespace}/{job} - Schedule a new event
/// - PUT /v1/schedule/{namespace}/{job}/{event_id}/completed - Set state as completed for event
/// - PUT /v1/schedule/{namespace}/{job}/{event_id}/disable - Set state as disabled for event
fn router() -> Router<Body, Infallible> {
    Router::build().middleware(Middleware::pre(logger))
}
