#[derive(Debug, Clone)]
pub struct Event {
    key: String,
    value: serde_json::Value,
    id: uuid::Uuid,
    idempotence_key: uuid::Uuid,
    state: State,
    namespace: String,
    job_type: String,
    created_at: chrono::DateTime<chrono::Utc>,
    scheduled_at: chrono::DateTime<chrono::Utc>,
}

impl Event {
    pub fn new(
        key: String,
        namespace: String,
        job_type: String,
        schedule_at: chrono::DateTime<chrono::Utc>,
        value: Option<serde_json::Value>,
    ) -> Event {
        let value: serde_json::Value = value.unwrap_or(serde_json::Value::Null);

        Event {
            key,
            value,
            id: uuid::Uuid::new_v4(),
            idempotence_key: uuid::Uuid::new_v4(),
            state: State::Scheduled,
            namespace,
            job_type,
            created_at: chrono::Utc::now(),
            scheduled_at: schedule_at,
        }
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
            idempotence_key: uuid::Uuid::new_v4(),
            state: State::Scheduled,
            namespace: self.namespace.clone(),
            job_type: self.job_type.clone(),
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

#[derive(Debug, Copy, Clone)]
enum State {
    Scheduled,
    Disabled,
    Completed,
}
