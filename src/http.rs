pub mod event {
    use serde::Deserialize;
    use serde_json::json;
    use tide::Request;

    use crate::{
        db::event::EventRepoPgsql,
        event::{Event, State},
        search::SearchQuery,
    };

    pub async fn schedule_event(mut req: Request<EventRepoPgsql>) -> tide::Result {
        let event: CreateEvent = req.body_json().await?;

        let repo: &EventRepoPgsql = req.state();
        match repo.insert(event.clone()).await {
            Ok(event) => Ok(tide::Response::builder(200)
                .body(serde_json::to_string(&event).unwrap())
                .build()),
            Err(e) => {
                let (code, msg): (u16, &str) = match e {
                    crate::db::event::RepoErr::AlreadyScheduled => (409, "Unable to insert"),
                    _ => (500, "Internal Server Error"),
                };
                let err = tide::Error::from_str(code, msg);
                tide::Result::Err(err)
            }
        }
    }

    pub async fn search_events(mut req: Request<EventRepoPgsql>) -> tide::Result {
        let query: SearchQuery = req.body_json().await?;
        let repo: &EventRepoPgsql = req.state();
        let events: Vec<Event> = repo.search(&query).await?;
        let (min, max) = query.scheduled_at().into_inner();
        let body = json!({
            "namespace": query.namespace(),
            "state": query.state(),
            "scheduletAtMin": min,
            "scheduledAtMax": max,
            "limit": query.limit(),
            "events": events
        });

        ok(200, body)
    }

    pub async fn settle_event(mut req: Request<EventRepoPgsql>) -> tide::Result {
        let update: SettleEvent = req.body_json().await?;
        let repo: &EventRepoPgsql = req.state();
        let res = repo.change_state(&update).await;

        match res {
            Ok(Some(event)) => ok(200, serde_json::to_string(&event).unwrap()),
            Ok(None) => match repo.get(&update.key, update.id, &update.namespace).await {
                Ok(Some(event)) => ok(200, serde_json::to_string(&event).unwrap()),
                Ok(None) => err(404, "Event not found"),
                Err(e) => match e {
                    crate::db::event::RepoErr::Connection => todo!(),
                    crate::db::event::RepoErr::AlreadyScheduled => todo!(),
                    crate::db::event::RepoErr::IllegalState => todo!(),
                    crate::db::event::RepoErr::Conversion => todo!(),
                    crate::db::event::RepoErr::NoResult => todo!(),
                    crate::db::event::RepoErr::Other(_) => todo!(),
                    crate::db::event::RepoErr::Unknown => todo!(),
                },
            },
            Err(e) => match e {
                crate::db::event::RepoErr::Connection => todo!(),
                crate::db::event::RepoErr::AlreadyScheduled => todo!(),
                crate::db::event::RepoErr::IllegalState => err(409, ""),
                crate::db::event::RepoErr::Conversion => todo!(),
                crate::db::event::RepoErr::NoResult => todo!(),
                crate::db::event::RepoErr::Other(_) => todo!(),
                crate::db::event::RepoErr::Unknown => err(500, ""),
            },
        }
    }

    pub async fn settle_and_next(mut req: Request<EventRepoPgsql>) -> tide::Result {
        let settle: SettleAndNextEvent = req.body_json().await?;
        let repo: &EventRepoPgsql = req.state();
        match repo.update_and_insert(&settle).await {
            Ok(event) => ok(200, serde_json::to_string(&event).unwrap()),
            Err(_) => err(400, "Unable to perform settle and schedule"),
        }
    }

    fn ok<S, M>(status: S, msg: M) -> tide::Result
    where
        S: TryInto<tide::StatusCode>,
        S::Error: std::fmt::Debug,
        M: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        let res = tide::Response::builder(status)
            .body(msg.to_string())
            .build();

        Ok(res)
    }

    fn err<S, M>(status: S, msg: M) -> tide::Result
    where
        S: TryInto<tide::StatusCode>,
        S::Error: std::fmt::Debug,
        M: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        tide::Result::Err(tide::Error::from_str(status, msg))
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct SettleEvent {
        pub key: String,
        pub id: uuid::Uuid,
        pub namespace: String,
        pub state: State,
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
            let timestamp =
                chrono::DateTime::parse_from_rfc3339(self.schedule_at.as_str()).unwrap();
            Ok(timestamp.with_timezone(&chrono::Utc))
        }
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct SettleAndNextEvent {
        pub id: uuid::Uuid,
        pub state: State,
        pub next: CreateEvent,
    }
}

pub mod webhook {}
