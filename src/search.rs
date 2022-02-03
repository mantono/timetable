use std::{convert::Infallible, ops::RangeInclusive, sync::Arc, vec};

use chrono::TimeZone;
use postgres_types::ToSql;
use serde_derive::Deserialize;
use tokio_postgres::{GenericClient, Row};

use crate::{
    db::event::EventRepoPgsql,
    event::{Event, State},
};

#[derive(Deserialize, Debug, Clone)]
pub struct SearchQuery {
    namespace: String,
    key: Option<String>,
    state: Option<Vec<State>>,
    order: Option<Order>,
    limit: Option<u32>,
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

    pub fn limit(&self) -> u32 {
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
