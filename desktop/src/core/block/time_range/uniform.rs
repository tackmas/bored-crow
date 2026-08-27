
use chrono::{Weekday, WeekdaySet};

use serde::{Deserialize, Serialize};

use super::{TimeRange, WeekScheduleT};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UniformWeekdays {
    pub time_range: TimeRange,
    #[serde(with = "super::weekday_set_serde")]
    pub enabled_weekdays: WeekdaySet,
}

impl UniformWeekdays {
    pub fn from_parts(time_range: TimeRange, enabled_weekdays: WeekdaySet) -> Self {
        Self {
            time_range,
            enabled_weekdays
        }
    }
}

impl WeekScheduleT for UniformWeekdays {
    fn enabled_weekdays(&self) -> WeekdaySet {
        self.enabled_weekdays
    }
    fn time_range_on_weekday(&self, weekday: Weekday) -> Option<&TimeRange> {
        self.enabled_weekdays
            .contains(weekday)
            .then_some(&self.time_range)
    }
}