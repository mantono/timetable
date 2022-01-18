use std::{convert::Infallible, net::SocketAddr};

use clap::Args;
use clap::Parser;
use event::Event;
use hyper::{Body, Request, Response, Server, StatusCode};
use routerify::{ext::RequestExt, Middleware, RequestInfo, Router, RouterService};
use search::Order;

use crate::config::Config;

mod config;
mod event;
mod search;

#[tokio::main]
async fn main() {
    let cfg: Config = Config::parse();
    println!("{}", cfg.db_url());
    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let service = RouterService::new(router).unwrap();

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Create a server by passing the created service to `.serve` method.
    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}

/// - POST /v1/schedule/search - Search for events
/// - PUT /v1/schedule - Schedule event
/// - PUT /v1/schedule/{namespace}/{event_id}/{state} - Update the state of the event
fn router() -> Router<Body, Infallible> {
    Router::builder()
        .middleware(Middleware::pre(logger))
        .post("/v1/schedule/search", search_events)
        .put("/v1/schedule", schedule_event)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap()
}

// A handler for "/" page.
async fn search_events(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Access the app state.
    // let state = req.data::<State>().unwrap();
    // println!("State value: {}", state.0);

    Ok(Response::new(Body::from("Hello world!")))
}

async fn schedule_event(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello world!")))
}

// Define an error handler function which will accept the `routerify::Error`
// and the request information and generates an appropriate response.
async fn error_handler(err: routerify::RouteError, _: RequestInfo) -> Response<Body> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

// A middleware which logs an http request.
async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!(
        "{} {} {}",
        req.remote_addr(),
        req.method(),
        req.uri().path()
    );
    Ok(req)
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
