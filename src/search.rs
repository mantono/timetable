use std::{convert::Infallible, ops::RangeInclusive, vec};

use chrono::TimeZone;
use serde_derive::Deserialize;
use tokio_postgres::Row;

use crate::event::{Event, State};

#[derive(Deserialize, Debug, Clone)]
pub struct SearchQuery {
    namespace: String,
    key: Option<String>,
    state: Option<Vec<State>>,
    order: Option<Order>,
    limit: Option<usize>,
    scheduled_at_min: Option<chrono::DateTime<chrono::Utc>>,
    scheduled_at_max: Option<chrono::DateTime<chrono::Utc>>,
}

impl SearchQuery {
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn key(&self) -> &Option<String> {
        &self.key
    }

    pub fn state(&self) -> Vec<State> {
        match &self.state {
            Some(state) => state.clone(),
            None => vec![State::Scheduled],
        }
    }

    pub fn order(&self) -> Order {
        self.order.unwrap_or(Order::Asc)
    }

    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(100)
    }

    pub fn scheduled_at(&self) -> RangeInclusive<chrono::DateTime<chrono::Utc>> {
        let epoch = chrono::Utc.ymd(1970, 1, 1).and_hms(0, 0, 0);
        let start: chrono::DateTime<chrono::Utc> = self.scheduled_at_min.unwrap_or(epoch);

        let end: chrono::DateTime<chrono::Utc> =
            self.scheduled_at_max.unwrap_or(chrono::Utc::now());

        start..=end
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum Order {
    #[serde(alias = "ASCENDING")]
    Asc,
    #[serde(alias = "DESCENDING")]
    Desc,
    #[serde(alias = "RANDOM")]
    Rand,
}

pub struct EventService {
    repo: EventRepoPgsql,
}

impl EventService {
    pub fn new(repo: EventRepoPgsql) -> EventService {
        EventService { repo }
    }

    pub fn search(&self, query: SearchQuery) -> Result<Vec<&Event>, Infallible> {
        Ok(vec![])
    }

    pub fn insert(&mut self, event: Event) -> Result<Event, Infallible> {
        Ok(event)
    }

    pub fn get(&self, namespace: &str, event_id: uuid::Uuid) -> Result<Option<Event>, Infallible> {
        Ok(None)
    }

    pub fn change_state(
        &mut self,
        namespace: &str,
        event_id: uuid::Uuid,
        state: State,
    ) -> Result<Event, Infallible> {
        todo!("")
    }
}

/* pub trait EventRepo {
    type Error;

    fn init(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn search(&self, query: SearchQuery) -> Result<Vec<&Event>, Self::Error>;
    fn insert(&mut self, event: Event) -> Result<Event, Self::Error>;

    fn get(&self, namespace: &str, event_id: uuid::Uuid) -> Result<Option<Event>, Self::Error>;

    fn change_state(
        &mut self,
        namespace: &str,
        event_id: uuid::Uuid,
        new_state: State,
    ) -> Result<(), Self::Error>;
} */

pub enum EventRepoErr {
    Connection,
    InvalidState,
}

pub struct EventRepoPgsql {
    client: tokio_postgres::Client,
}

impl EventRepoPgsql {
    pub fn new(client: tokio_postgres::Client) -> EventRepoPgsql {
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
                .simple_query("CREATE TYPE state AS ENUM('SCHEDULED', 'DISABLED', 'COMPLETED');")
                .await
                .map(|_| ()),
            _ => panic!("Bad database state for created enums"),
        }
    }

    async fn init_table(&self) -> Result<(), tokio_postgres::Error> {
        self.client.simple_query("
            CREATE TABLE IF NOT EXISTS events(
                id                     UUID                            NOT NULL PRIMARY KEY DEFAULT uuid_generate_v4(),
                key                    VARCHAR(128)                    NOT NULL,
                value                  JSON                            NOT NULL DEFAULT '{}'::json,
                idempotence_key        UUID                            NOT NULL UNIQUE DEFAULT uuid_generate_v4(),
                namespace              VARCHAR(64)                     NOT NULL,
                state                  state                           NOT NULL DEFAULT 'SCHEDULED',
                created_at             TIMESTAMP WITH TIME ZONE        NOT NULL DEFAULT CURRENT_TIMESTAMP,
                scheduled_at           TIMESTAMP WITH TIME ZONE        NOT NULL
            );
        ").await.map(|_| ())
    }

    async fn init_idx(&self) -> Result<(), tokio_postgres::Error> {
        self.client.simple_query("
            CREATE INDEX IF NOT EXISTS key_idx ON events(namespace, key);
            CREATE INDEX IF NOT EXISTS state_idx ON events(namespace, scheduled_at, state);
            CREATE UNIQUE INDEX IF NOT EXISTS single_scheduled_idx ON events(namespace, key) WHERE state = 'SCHEDULED';
        ").await.map(|_| ())
    }

    fn search(&self, query: SearchQuery) -> Result<Vec<&Event>, Infallible> {
        todo!()
    }

    fn insert(&mut self, event: Event) -> Result<Event, Infallible> {
        todo!()
    }

    fn get(&self, namespace: &str, event_id: uuid::Uuid) -> Result<Option<Event>, Infallible> {
        todo!()
    }

    fn change_state(
        &mut self,
        namespace: &str,
        event_id: uuid::Uuid,
        new_state: State,
    ) -> Result<(), Infallible> {
        todo!()
    }
}

struct VecRepo(Vec<Event>);
/*
impl EventRepo for VecRepo {
    type Error = Infallible;

    fn search(&self, query: SearchQuery) -> Result<Vec<&Event>, Self::Error> {
        let events: Vec<&Event> = self
            .0
            .iter()
            .filter(|ev| query.scheduled_at().contains(ev.schedule_at()))
            .filter(|ev| query.state().contains(&ev.state()))
            .filter(|ev| match query.key() {
                Some(key) => ev.key() == key,
                None => true,
            })
            .collect();

        Ok(events)
    }

    fn insert(&mut self, event: Event) -> Result<Event, Self::Error> {
        let row: Option<Event> = self.get(event.namespace(), event.id())?;
        match row {
            Some(event) => Ok(event),
            None => {
                self.0.push(event.clone());
                Ok(event)
            }
        }
    }

    fn get(&self, namespace: &str, event_id: uuid::Uuid) -> Result<Option<Event>, Self::Error> {
        let x = self
            .0
            .iter()
            .filter(|ev| ev.id() == event_id)
            .map(|ev| ev.clone())
            .next();

        Ok(x)
    }

    fn change_state(
        &mut self,
        namespace: &str,
        event_id: uuid::Uuid,
        new_state: State,
    ) -> Result<(), Self::Error> {
        todo!()
    }
} */
