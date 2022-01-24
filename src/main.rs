use std::{convert::Infallible, net::SocketAddr};

use clap::Args;
use clap::Parser;
use event::Event;
use search::Order;
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use tide::Request;
use tokio_postgres::GenericClient;
use tokio_postgres::{Error, NoTls};

use crate::config::Config;
use crate::search::EventRepoPgsql;

mod config;
mod event;
mod search;

#[tokio::main]
async fn main() {
    let cfg: Config = Config::parse();
    println!("{}", cfg.db_url());

    let (client, connection) = tokio_postgres::connect(cfg.db_url(), NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let repo = EventRepoPgsql::new(client);
    repo.init().await.unwrap();

    let mut app = tide::with_state(repo);
    app.at("/v1/schedule").put(schedule_event);
    let bind: String = format!("127.0.0.1:{}", 3000);
    app.listen(&bind).await.unwrap();
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateEvent {
    key: String,
    value: Option<serde_json::Value>,
    namespace: String,
    #[serde(alias = "scheduleAt")]
    schedule_at: String,
}

impl CreateEvent {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> serde_json::Value {
        self.value.clone().unwrap_or(serde_json::Value::Null)
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn schedule_at(&self) -> Result<chrono::DateTime<chrono::Utc>, String> {
        let timestamp = chrono::DateTime::parse_from_rfc3339(self.schedule_at.as_str()).unwrap();
        Ok(timestamp.with_timezone(&chrono::Utc))
    }
}

async fn schedule_event(mut req: Request<EventRepoPgsql>) -> tide::Result {
    let event: CreateEvent = req.body_json().await.unwrap();
    let repo: &EventRepoPgsql = req.state();
    repo.insert(event.clone()).await.unwrap();
    Ok("Hello world!".into())
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
