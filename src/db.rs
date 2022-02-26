pub mod event {
    use std::sync::Arc;

    use log::info;
    use sqlx::{pool::PoolConnection, postgres::PgPoolOptions, Executor, FromRow, Postgres};

    use crate::{
        event::{Event, State},
        http::event::{CreateEvent, SettleAndNextEvent, SettleEvent},
        search::SearchQuery,
    };

    #[derive(Clone)]
    pub struct EventRepoPgsql {
        pool: Arc<sqlx::PgPool>,
    }

    impl EventRepoPgsql {
        pub async fn new(db_url: &str) -> Result<EventRepoPgsql, sqlx::Error> {
            let pool = PgPoolOptions::new()
                .max_connections(2)
                .connect(db_url)
                .await?;

            let repo = EventRepoPgsql {
                pool: Arc::new(pool),
            };

            Ok(repo)
        }

        pub async fn init(&mut self) -> Result<(), sqlx::Error> {
            self.init_enum().await?;
            self.init_table().await?;
            self.init_idx().await
        }

        /*         async fn insert_stmt(&mut self) -> Result<&Statement, sqlx::Error> {
            match &self.insert_stmt {
                Some(stmt) => Ok(stmt),
                None => {
                    let stmt: Statement = self
                        .client
                        .prepare(include_str!("../res/db/insert_event.sql"))
                        .await?;

                    self.insert_stmt = Some(stmt);
                    Ok(&stmt)
                }
            }
        } */
        async fn conn(&self) -> Result<PoolConnection<Postgres>, sqlx::Error> {
            self.pool.acquire().await
        }

        async fn init_enum(&self) -> Result<(), sqlx::Error> {
            let mut rows: Vec<_> = sqlx::query(
                "SELECT * FROM pg_enum WHERE enumlabel IN ('SCHEDULED', 'DISABLED', 'COMPLETED')",
            )
            .map(|_| ())
            .fetch_all(&mut self.conn().await?)
            .await?;

            /*
            let rows: Vec<Row> = self
            .client
            .query(
                "SELECT * FROM pg_enum WHERE enumlabel IN ('SCHEDULED', 'DISABLED', 'COMPLETED')",
                &vec![],
            )
            .await?; */

            match rows.len() {
                3 => Ok(()),
                0 => self
                    .conn()
                    .await?
                    .execute(include_str!("../res/db/create_state_enums.sql"))
                    .await
                    .map(|_| ()),
                _ => panic!("Bad database state for created enums"),
            }
        }

        async fn init_table(&self) -> Result<(), sqlx::Error> {
            self.conn()
                .await?
                .execute(include_str!("../res/db/create_events_table.sql"))
                .await
                .map(|_| ())
        }

        async fn init_idx(&self) -> Result<(), sqlx::Error> {
            self.conn()
                .await?
                .execute(include_str!("../res/db/create_events_indices.sql"))
                .await
                .map(|_| ())
        }

        pub async fn search(&self, query: &SearchQuery) -> Result<Vec<Event>, sqlx::Error> {
            info!("0");

            /*             let states: Vec<State> = query.state();
            let (min, max) = query.scheduled_at().into_inner();

            let params: [&(dyn ToSql + Sync); 8] = [
                &query.namespace(),
                &query.key(),
                &states.get(0),
                &states.get(1),
                &states.get(2),
                &min,
                &max,
                &query.limit(),
            ];

            info!("1");

            let rows: Vec<Row> = self
                .client
                .query(
                    include_str!("../res/db/search_events.sql"),
                    params.as_slice(),
                )
                .await?;

            info!("2");

            let events: Vec<Event> = rows
                .iter()
                .filter_map(|row| Event::try_from(row).ok())
                .collect();

            info!("Search successful"); */

            let events = vec![];

            Ok(events)
        }

        pub async fn insert(&self, event: CreateEvent) -> Result<Event, RepoErr> {
            let event: Event = sqlx::query(include_str!("../res/db/insert_event.sql"))
                .bind(event.key())
                .bind(event.namespace())
                .bind(event.schedule_at().unwrap())
                .bind(event.value())
                .try_map(|row| Event::from_row(&row))
                .fetch_one(&mut self.conn().await?)
                .await?;

            Ok(event)
        }

        pub async fn get(
            &self,
            key: &str,
            id: uuid::Uuid,
            namespace: &str,
        ) -> Result<Event, RepoErr> {
            let event: Option<Event> =
                sqlx::query("SELECT * FROM events WHERE key = ? AND id = ? AND namespace = ?")
                    .bind(key)
                    .bind(id)
                    .bind(namespace)
                    .try_map(|row| Event::from_row(&row))
                    .fetch_optional(&mut self.conn().await?)
                    .await?;

            match event {
                Some(event) => Ok(event),
                None => Err(RepoErr::NoResult),
            }
        }

        pub async fn change_state(&self, update: &SettleEvent) -> Result<Event, RepoErr> {
            if let State::Scheduled = update.state {
                return Err(RepoErr::IllegalState);
            }

            let SettleEvent {
                key,
                id,
                namespace,
                state,
            } = update;

            let event: Option<Event> = sqlx::query(include_str!("../res/db/update_event.sql"))
                .bind(key)
                .bind(id)
                .bind(namespace)
                .bind(state)
                .try_map(|row| Event::from_row(&row))
                .fetch_optional(&mut self.conn().await?)
                .await?;

            match event {
                Some(event) => Ok(event),
                None => Err(RepoErr::NoResult),
            }
        }
        /*
        fn parse_row(row: Result<PgRow, sqlx::Error>) -> Result<Event, RepoErr> {
            match Event::try_from(row) {
                Ok(event) => Ok(Some(event)),
                Err(e) => Err(RepoErr::Other(e.to_string())),
            }
        }

        fn parse_row_opt(
            row: Result<std::option::Option<PgRow>, sqlx::Error>,
        ) -> Result<Event, RepoErr> {
            let row: Option<PgRow> = row?;
            match row {
                Some(row) => match Event::try_from(row) {
                    Ok(event) => Ok(Some(event)),
                    Err(e) => Err(RepoErr::Other(e.to_string())),
                },
                None => Err(RepoErr::NoResult),
            }
        } */

        pub async fn update_and_insert(
            &self,
            replace: &SettleAndNextEvent,
        ) -> Result<Event, RepoErr> {
            if let State::Scheduled = replace.state {
                return Err(RepoErr::IllegalState);
            }

            let SettleAndNextEvent { id, state, next } = replace;

            /*             let params: [&(dyn ToSql + Sync); 6] = [
                &state,
                &id,
                &next.key(),
                &next.namespace(),
                &next.schedule_at().unwrap(),
                &next.value(),
            ]; */

            //self.client.borrow_mut().transaction();

            /*             let mut cln: tokio::sync::MutexGuard<tokio_postgres::Client> = self.client.lock().await;
            let tx = cln.transaction().await?;
            let rows: Vec<Row> = tx
                .query(
                    include_str!("../res/db/update_and_insert.sql"),
                    params.as_slice(),
                )
                .await?; */
            //let mut client = self.client.lock().await;

            let event: Option<Event> = sqlx::query("../res/db/update_and_insert.sql")
                .bind(state)
                .bind(id)
                .bind(next.key())
                .bind(next.namespace())
                .bind(next.schedule_at().unwrap())
                .bind(next.value())
                .try_map(|row| Event::from_row(&row))
                .fetch_optional(&mut self.conn().await?)
                .await?;

            /*             let rows: Vec<Row> = self
            .client
            .lock()
            .await
            .transaction()
            .await?
            .query(
                include_str!("../res/db/update_and_insert.sql"),
                params.as_slice(),
            )
            .await?; */

            match event {
                Some(event) => Ok(event),
                None => Err(RepoErr::NoResult),
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

    impl From<sqlx::Error> for RepoErr {
        fn from(e: sqlx::Error) -> Self {
            log::error!("DB Error: {:?}", e);
            match e {
                sqlx::Error::Database(db_err) => match db_err.constraint() {
                    Some(_) => RepoErr::AlreadyScheduled,
                    None => RepoErr::Connection,
                },
                sqlx::Error::Io(_) => RepoErr::Connection,
                _ => RepoErr::Unknown,
            }
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
