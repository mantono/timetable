use clap::Parser;
use std::process;

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

#[async_std::main]
async fn main() -> tide::Result<()> {
    let cfg: Config = Config::parse();
    println!("{}", cfg.db_url());
    setup_logging(&cfg.verbosity());

    let mut repo: EventRepoPgsql = match EventRepoPgsql::new(cfg.db_url()).await {
        Ok(repo) => repo,
        Err(e) => {
            log::error!("Failed to init repo: {}", e);
            process::exit(1)
        }
    };

    repo.init().await.unwrap();

    let mut app = tide::with_state(repo);
    app.at("/v1/schedule").put(schedule_event);
    app.at("/v1/schedule/settle").put(settle_event);
    app.at("/v1/schedule/next").put(settle_and_next);
    app.at("/v1/schedule/search").post(search_events);
    let bind: String = format!("127.0.0.1:{}", 3000);

    app.listen(&bind).await.unwrap();

    Ok(())
}
