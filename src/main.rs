use event::Event;
use search::Order;

mod event;
mod search;

#[tokio::main]
async fn main() {}

/// - POST /v1/schedule/search - Search for events
/// - PUT /v1/schedule - Schedule event
/// - PUT /v1/schedule/{namespace}/{event_id}/{state} - Update the state of the event
fn router() -> Router<Body, Infallible> {
    Router::build().middleware(Middleware::pre(logger))
}

pub struct CreateEventReq {
    namespace: String,
    event: Event,
}

pub struct WebHookReq {
    namespace: String,
    url: String,
    interval: Option<chrono::Duration>,
    limit: Option<usize>,
    order: Option<Order>,
}

pub struct WebHook {
    namespace: String,
    url: String,
    interval: chrono::Duration,
    limit: usize,
    order: Order,
}

impl From<WebHookReq> for WebHook {
    fn from(req: WebHookReq) -> Self {
        WebHook {
            namespace: req.namespace,
            url: req.url,
            interval: req.interval.unwrap_or(chrono::Duration::minutes(20)),
            limit: req.limit.unwrap_or(100),
            order: req.order.unwrap_or(Order::Asc),
        }
    }
}
