use std::sync::Arc;

use clap::Parser;
use tokio::sync::Mutex;
use tokio_postgres::NoTls;

use crate::config::Config;
use crate::db::event::EventRepoPgsql;
use crate::http::event::{schedule_event, search_events, settle_and_next, settle_event};
use crate::logger::setup_logging;

mod config;
mod db;
mod event;
mod http;
mod logger;
mod search;
mod webhook;

#[tokio::main]
async fn main() {
    let cfg: Config = Config::parse();
    println!("{}", cfg.db_url());
    setup_logging(&cfg.verbosity());

    let (client, con0) = tokio_postgres::connect(cfg.db_url(), NoTls).await.unwrap();
    let (client_trx, con1) = tokio_postgres::connect(cfg.db_url(), NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = con0.await {
            eprintln!("connection error: {}", e);
        }
    });

    tokio::spawn(async move {
        if let Err(e) = con1.await {
            eprintln!("connection error: {}", e);
        }
    });

    let client = Arc::new(client);
    let client_trx = Arc::new(Mutex::new(client_trx));
    let mut repo = EventRepoPgsql::new(client, client_trx).await.unwrap();
    repo.init().await.unwrap();

    let mut app = tide::with_state(repo);
    app.at("/v1/schedule").put(schedule_event);
    app.at("/v1/schedule/settle").put(settle_event);
    app.at("/v1/schedule/next").put(settle_and_next);
    app.at("/v1/schedule/search").post(search_events);
    let bind: String = format!("127.0.0.1:{}", 3000);
    app.listen(&bind).await.unwrap();
}
