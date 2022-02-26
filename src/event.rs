use std::{convert::Infallible, str::FromStr};

use postgres_types::{FromSql, ToSql};
use serde_derive::{Deserialize, Serialize};
use sqlx::{
    postgres::{PgRow, PgTypeInfo, PgValue, PgValueRef},
    Database, Decode, FromRow, Postgres, Row, Type, TypeInfo, Value,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    key: String,
    value: serde_json::Value,
    id: uuid::Uuid,
    namespace: String,
    #[serde(rename = "idempotenceKey")]
    idempotence_key: uuid::Uuid,
    state: State,
    #[serde(rename = "createdAt")]
    created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "scheduledAt")]
    scheduled_at: chrono::DateTime<chrono::Utc>,
}

impl Event {
    pub fn new(
        key: String,
        namespace: String,
        schedule_at: chrono::DateTime<chrono::Utc>,
        value: Option<serde_json::Value>,
    ) -> Event {
        let value: serde_json::Value = value.unwrap_or(serde_json::Value::Null);

        Event {
            key,
            value,
            id: uuid::Uuid::new_v4(),
            namespace,
            idempotence_key: uuid::Uuid::new_v4(),
            state: State::Scheduled,
            created_at: chrono::Utc::now(),
            scheduled_at: schedule_at,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &serde_json::Value {
        &self.value
    }

    pub fn id(&self) -> uuid::Uuid {
        self.id
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn idempotence_key(&self) -> uuid::Uuid {
        self.idempotence_key
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn schedule_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.scheduled_at
    }

    pub fn next(
        self,
        schedule_at: chrono::DateTime<chrono::Utc>,
        value: Option<serde_json::Value>,
    ) -> (Event, Event) {
        let value: serde_json::Value = value.unwrap_or(self.value.clone());

        let next = Event {
            key: self.key.clone(),
            id: uuid::Uuid::new_v4(),
            namespace: self.namespace.clone(),
            idempotence_key: uuid::Uuid::new_v4(),
            state: State::Scheduled,
            created_at: chrono::Utc::now(),
            scheduled_at: schedule_at,
            value,
        };

        (self.disable(), next)
    }

    pub fn next_duration(
        self,
        duration: chrono::Duration,
        value: Option<serde_json::Value>,
    ) -> (Event, Event) {
        let schedule_at: chrono::DateTime<chrono::Utc> = self.scheduled_at + duration;
        self.next(schedule_at, value)
    }

    pub fn is_scheduled(&self) -> bool {
        match self.state {
            State::Scheduled => true,
            _ => false,
        }
    }

    pub fn disable(self) -> Event {
        match self.state {
            State::Scheduled => self.change_state(State::Disabled),
            _ => self,
        }
    }

    pub fn complete(self) -> Event {
        match self.state {
            State::Scheduled | State::Disabled => self.change_state(State::Completed),
            _ => self,
        }
    }

    fn change_state(self, state: State) -> Event {
        Event { state, ..self }
    }
}

/* impl TryFrom<&Row> for Event {
    type Error = tokio_postgres::Error;

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

/* impl TryFrom<PgRow> for Event {
    type Error = Infallible;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        Event {
            key: value.get("key"),
            value: value.get("value"),
            id: value.get("id"),
            namespace: value.get("namespace"),
            idempotence_key: value.get("namespace"),
            state: value.get("state"),
            created_at: value.get("created_at"),
            scheduled_at: value.get("schedulet_at"),
        }
    }
} */

impl<'r> FromRow<'r, sqlx::postgres::PgRow> for Event {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        let state: State = row.get("state");
        let event = Event {
            key: row.get("key"),
            value: row.get("value"),
            id: row.get("id"),
            namespace: row.get("namespace"),
            idempotence_key: row.get("idempotence_key"),
            state,
            created_at: row.get("created_at"),
            scheduled_at: row.get("schedulet_at"),
        };

        Ok(event)
    }
}

#[derive(Serialize, Deserialize, ToSql, FromSql, Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "state", rename_all = "uppercase")]
#[postgres(name = "state")]
pub enum State {
    #[serde(alias = "SCHEDULED")]
    #[postgres(name = "SCHEDULED")]
    Scheduled,
    #[serde(alias = "DISABLED")]
    #[postgres(name = "DISABLED")]
    Disabled,
    #[serde(alias = "COMPLETED")]
    #[postgres(name = "COMPLETED")]
    Completed,
}

impl FromStr for State {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: String = s.to_uppercase();
        match s.as_str() {
            "SCHEDULED" => Ok(State::Scheduled),
            "DISABLED" => Ok(State::Disabled),
            "COMPLETED" => Ok(State::Completed),
            _ => Err("Invalid state value"),
        }
    }
}

/* impl Type<Postgres> for State {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::with_name("ENUM")
    }
}

impl<'r, DB: Database> sqlx::Decode<'r, DB> for State
where
    &'r str: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <&str as sqlx::Decode<DB>>::decode(value)?;
        let state = State::from_str(value)?;
        Ok(state)
    }
}

impl<'r, DB: Database> sqlx::Encode<'r, DB> for State {
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.encode(buf)
    }
} */
