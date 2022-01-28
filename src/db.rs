pub mod event {
    use std::{convert::Infallible, sync::Arc};

    use postgres_types::{FromSql, ToSql};
    use tokio_postgres::{error::DbError, Row};

    use crate::{
        event::{Event, State},
        http::event::{CreateEvent, UpdateEvent},
        search::SearchQuery,
    };

    #[derive(Clone)]
    pub struct EventRepoPgsql {
        client: Arc<tokio_postgres::Client>,
    }

    impl EventRepoPgsql {
        pub fn new(client: Arc<tokio_postgres::Client>) -> EventRepoPgsql {
            EventRepoPgsql { client }
        }

        pub async fn init(&self) -> Result<(), tokio_postgres::Error> {
            self.init_enum().await?;
            self.init_table().await?;
            self.init_idx().await
        }

        async fn init_enum(&self) -> Result<(), tokio_postgres::Error> {
            let rows: Vec<Row> = self
            .client
            .query(
                "SELECT * FROM pg_enum WHERE enumlabel IN ('SCHEDULED', 'DISABLED', 'COMPLETED')",
                &vec![],
            )
            .await?;

            match rows.len() {
                3 => Ok(()),
                0 => self
                    .client
                    .simple_query(include_str!("../res/db/create_state_enums.sql"))
                    .await
                    .map(|_| ()),
                _ => panic!("Bad database state for created enums"),
            }
        }

        async fn init_table(&self) -> Result<(), tokio_postgres::Error> {
            self.client
                .simple_query(include_str!("../res/db/create_events_table.sql"))
                .await
                .map(|_| ())
        }

        async fn init_idx(&self) -> Result<(), tokio_postgres::Error> {
            self.client
                .simple_query(include_str!("../res/db/create_events_indices.sql"))
                .await
                .map(|_| ())
        }

        fn search(&self, query: SearchQuery) -> Result<Vec<&Event>, Infallible> {
            todo!()
        }

        pub async fn insert(&self, event: CreateEvent) -> Result<Event, RepoErr> {
            let params: [&(dyn ToSql + Sync); 4] = [
                &event.key(),
                &event.namespace(),
                &event.schedule_at().unwrap(),
                &event.value(),
            ];

            let rows: Vec<Row> = self
                .client
                .query(
                    include_str!("../res/db/insert_event.sql"),
                    params.as_slice(),
                )
                .await?;

            match rows.first() {
                Some(row) => match row.try_into() {
                    Ok(event) => Ok(event),
                    Err(e) => Err(RepoErr::from(e)),
                },
                None => Err(RepoErr::NoResult),
            }
        }

        pub async fn get(
            &self,
            key: &str,
            id: uuid::Uuid,
            namespace: &str,
        ) -> Result<Option<Event>, RepoErr> {
            let params: [&(dyn ToSql + Sync); 3] = [&key, &id, &namespace];

            let rows: Vec<Row> = self
                .client
                .query(
                    "SELECT * FROM events WHERE key = $1 AND id = $2 AND namespace = $3",
                    params.as_slice(),
                )
                .await?;

            match rows.first() {
                Some(row) => match Event::try_from(row) {
                    Ok(event) => Ok(Some(event)),
                    Err(e) => Err(RepoErr::from(e)),
                },
                None => Ok(None),
            }
        }

        pub async fn change_state(&self, update: &UpdateEvent) -> Result<Option<Event>, RepoErr> {
            if let State::Scheduled = update.state {
                return Err(RepoErr::IllegalState);
            }

            let UpdateEvent {
                key,
                id,
                namespace,
                state,
            } = update;

            let params: [&(dyn ToSql + Sync); 4] = [&state, &id, &key, &namespace];

            let rows: Vec<Row> = self
                .client
                .query(
                    include_str!("../res/db/update_event.sql"),
                    params.as_slice(),
                )
                .await?;

            match rows.first() {
                Some(row) => match Event::try_from(row) {
                    Ok(event) => Ok(Some(event)),
                    Err(e) => Err(RepoErr::from(e)),
                },
                None => Ok(None),
            }
        }
    }

    /*     impl TryFrom<&Row> for Event {
        type Error = RepoErr;

        fn try_from(value: &Row) -> Result<Self, Self::Error> {
            let event = Event {
                id: value.try_get(0)?,
                key: value.try_get(1)?,
                value: value.try_get(2)?,
                idempotence_key: value.try_get(3)?,
                namespace: value.try_get(4)?,
                state: value.try_get(5)?,
                created_at: value.try_get(6)?,
                scheduled_at: value.try_get(7)?,
            };

            Ok(event)
        }
    } */

    impl From<tokio_postgres::Error> for RepoErr {
        fn from(e: tokio_postgres::Error) -> Self {
            log::error!("DB Error: {:?}", e);
            let db_err: &DbError = match e.as_db_error() {
                Some(err) => err,
                None => return RepoErr::Unknown,
            };

            if let Some("single_scheduled_idx") = db_err.constraint() {
                return RepoErr::AlreadyScheduled;
            };

            RepoErr::Other(e.to_string())
        }
    }

    #[derive(Debug)]
    pub enum RepoErr {
        Connection,
        AlreadyScheduled,
        IllegalState,
        Conversion,
        NoResult,
        Other(String),
        Unknown,
    }
}

pub mod webhook {}
