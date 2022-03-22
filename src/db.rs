pub mod event {
    use std::sync::Arc;

    use log::info;
    use postgres_types::ToSql;
    use tokio::sync::Mutex;
    use tokio::sync::MutexGuard;
    use tokio_postgres::GenericClient;
    use tokio_postgres::{error::DbError, Client, Row, Statement, Transaction};

    use crate::{
        event::{Event, State},
        http::event::{CreateEvent, SettleAndNextEvent, SettleEvent},
        search::SearchQuery,
    };

    #[derive(Clone)]
    pub struct EventRepoPgsql {
        client: Arc<tokio_postgres::Client>,
        client_trx: Arc<Mutex<tokio_postgres::Client>>,
        insert_stmt: Statement,
        update_stmt: Statement,
        search_stmt: Statement,
    }

    impl EventRepoPgsql {
        pub async fn new(
            client: Arc<tokio_postgres::Client>,
            client_trx: Arc<Mutex<tokio_postgres::Client>>,
        ) -> Result<EventRepoPgsql, tokio_postgres::Error> {
            let insert_stmt = client
                .prepare(include_str!("../res/db/insert_event.sql"))
                .await?;

            let update_stmt = client
                .prepare(include_str!("../res/db/update_event.sql"))
                .await?;

            let search_stmt = client
                .prepare(include_str!("../res/db/search_events.sql"))
                .await?;

            let repo = EventRepoPgsql {
                client,
                client_trx,
                insert_stmt,
                update_stmt,
                search_stmt,
            };

            Ok(repo)
        }

        pub async fn init(&mut self) -> Result<(), tokio_postgres::Error> {
            self.init_enum().await?;
            self.init_table().await?;
            self.init_idx().await?;

            Ok(())
        }

        async fn init_enum(&self) -> Result<(), tokio_postgres::Error> {
            let rows: Vec<Row> = self.client
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

        pub async fn search(
            &self,
            query: &SearchQuery,
        ) -> Result<Vec<Event>, tokio_postgres::Error> {
            let states: Vec<State> = query.state();
            let (min, max) = query.scheduled_at();

            let limit: i64 = query.limit();

            let params: [&(dyn ToSql + Sync); 8] = [
                &query.namespace(),
                &query.key(),
                &states.get(0),
                &states.get(1),
                &states.get(2),
                &min,
                &max,
                &limit,
            ];

            let rows: Vec<Row> = self.client.query(&self.search_stmt, &params).await?;

            let events: Vec<Event> = rows
                .iter()
                .filter_map(|row| Event::try_from(row).ok())
                .collect();

            info!("Search successful");

            Ok(events)
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
                .query(&self.insert_stmt, params.as_slice())
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

        pub async fn change_state(&self, update: &SettleEvent) -> Result<Option<Event>, RepoErr> {
            if let State::Scheduled = update.state {
                return Err(RepoErr::IllegalState);
            }

            let SettleEvent {
                key,
                id,
                namespace,
                state,
            } = update;

            let params: [&(dyn ToSql + Sync); 4] = [&state, &id, &key, &namespace];

            let rows: Vec<Row> = self
                .client
                .query(&self.update_stmt, params.as_slice())
                .await?;

            match rows.first() {
                Some(row) => match Event::try_from(row) {
                    Ok(event) => Ok(Some(event)),
                    Err(e) => Err(RepoErr::from(e)),
                },
                None => Ok(None),
            }
        }

        pub async fn update_and_insert(
            &self,
            replace: &SettleAndNextEvent,
        ) -> Result<Event, RepoErr> {
            if let State::Scheduled = replace.state {
                return Err(RepoErr::IllegalState);
            }

            let SettleAndNextEvent {
                key,
                id,
                namespace,
                state,
                next,
            } = replace;

            let params: [&(dyn ToSql + Sync); 4] = [&state, &id, &key, &namespace];

            let mut client_trx = self.client_trx.lock().await;

            let trx: Transaction = client_trx.transaction().await?;

            trx.query(
                include_str!("../res/db/update_event.sql"),
                params.as_slice(),
            )
            .await?;

            let params: [&(dyn ToSql + Sync); 4] = [
                &key,
                &namespace,
                &next.schedule_at().unwrap(),
                &next.value(),
            ];

            let rows: Vec<Row> = trx
                .query(
                    include_str!("../res/db/insert_event.sql"),
                    params.as_slice(),
                )
                .await?;

            trx.commit().await?;
            drop(client_trx);

            match rows.first() {
                Some(row) => match Event::try_from(row) {
                    Ok(event) => Ok(event),
                    Err(e) => Err(RepoErr::from(e)),
                },
                None => Err(RepoErr::NoResult),
            }
        }
    }

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
