use std::ops::RangeInclusive;

use chrono::TimeZone;
use serde_derive::Deserialize;

use crate::event::State;

#[derive(Deserialize, Debug, Clone)]
pub struct SearchQuery {
    namespace: String,
    job_type: String,
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

    pub fn job_type(&self) -> &str {
        &self.job_type
    }

    pub fn state(&self) -> Vec<State> {
        self.state.unwrap_or(vec![State::Scheduled])
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

#[derive(Deserialize, Debug, Clone)]
pub enum Order {
    #[serde(alias = "ASCENDING")]
    Asc,
    #[serde(alias = "DESCENDING")]
    Desc,
    #[serde(alias = "RANDOM")]
    Rand,
}
