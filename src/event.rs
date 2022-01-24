use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    key: String,
    value: serde_json::Value,
    id: uuid::Uuid,
    namespace: String,
    idempotence_key: uuid::Uuid,
    state: State,
    created_at: chrono::DateTime<chrono::Utc>,
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    #[serde(alias = "SCHEDULED")]
    Scheduled,
    #[serde(alias = "DISABLED")]
    Disabled,
    #[serde(alias = "COMPLETED")]
    Completed,
}
