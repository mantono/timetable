pub mod event {
    use serde::Deserialize;
    use tide::Request;

    use crate::db::event::EventRepoPgsql;

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
}

pub mod webhook {}
