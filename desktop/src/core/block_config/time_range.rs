

use chrono::{
    self,
    NaiveTime,
    Datelike,
    TimeDelta,
    Weekday,
    WeekdaySet,
};

use tokio::time::{
    self,
    Duration
};

use crate::platform::Blocker;

use super::{
    Group,
};

pub struct TimeRange {
    start: NaiveTime,
    end: NaiveTime
}

impl TimeRange {
    pub fn unsigned_diff(&self) -> Duration {
        let mut diff = self.end - self.start;

        if diff < TimeDelta::zero() {
            diff += TimeDelta::hours(24);
        }

        diff
            .to_std()
            .unwrap()
    }
}

pub struct CustomWeekdays {
    pub monday: Option<TimeRange>,
    pub tuesday: Option<TimeRange>,
    pub wednesday: Option<TimeRange>,
    pub thursday: Option<TimeRange>,
    pub friday: Option<TimeRange>,
    pub saturday: Option<TimeRange>,
    pub sunday: Option<TimeRange>,
}

impl CustomWeekdays {
    fn time_range_with_weekday(&self, weekday: Weekday) -> &TimeRange {
        use Weekday::*;

        match weekday {
            Mon => &self.monday,
            Tue => &self.tuesday,
            Wed => &self.wednesday,
            Thu => &self.thursday,
            Fri => &self.friday,
            Sat => &self.saturday,
            Sun => &self.sunday,
        }
        .as_ref()
        .unwrap()
    }

    fn set_from_fields(&self) -> WeekdaySet {
        let arr = 
            [
                &self.monday,
                &self.tuesday,
                &self.wednesday,
                &self.thursday,
                &self.friday,
                &self.saturday,
                &self.sunday,
            ];

        arr
            .into_iter()
            .enumerate()
            .filter(|(_, day)| day.is_some())
            .map(|(i, _)| Weekday::try_from(i as u8).unwrap())
            .collect()
    }
}

pub struct UniformWeekdays {
    pub time_range: TimeRange,
    pub weekdays: chrono::WeekdaySet,
}   

pub enum WeekdayRuleMode {
    Custom(CustomWeekdays),
    Uniform(UniformWeekdays),
}

impl WeekdayRuleMode {
    fn weekday_set(&self) -> WeekdaySet {
        match self {
            WeekdayRuleMode::Custom(custom_weekday) => custom_weekday.set_from_fields(),
            WeekdayRuleMode::Uniform(uniform_weekdays) => uniform_weekdays.weekdays
        }
    }
    fn time_range(&self, weekday: Weekday) -> &TimeRange {
        match self {
            WeekdayRuleMode::Custom(custom_weekday) => custom_weekday.time_range_with_weekday(weekday),
            WeekdayRuleMode::Uniform(uniform_weekdays) => &uniform_weekdays.time_range
        }
    }
}


impl Group {
    pub async fn block_with_time_range(&self, weekday_rule_mode: WeekdayRuleMode, blocker: Blocker) {
        loop {
            let now = chrono::Local::now();
            let now_weekday = now.weekday();

            let set = weekday_rule_mode.weekday_set();

            let block_weekday = set
                .iter(now_weekday)
                .next()
                .expect("Should always be a weekday in the set");
            let time_range = weekday_rule_mode.time_range(block_weekday);

            let until_start = {
                let until_start_day = {
                    let day_diff = block_weekday.days_since(now_weekday);  

                    TimeDelta::days(day_diff as i64)
                };

                let until_start_clock = {
                    let start_clock = time_range.start;
                    start_clock.signed_duration_since(now.time())
                };

                until_start_day + until_start_clock
            };

            let until_start_std = until_start
                .abs()
                .to_std()
                .unwrap();

            let start_end_diff = time_range.unsigned_diff();

            if until_start.num_seconds() < 0 {
                if until_start_std < start_end_diff {
                    let until_end = start_end_diff - until_start_std;
                    self.block_until_unblock(until_end, blocker.clone()).await;
                } else {
                    let until_midnight = (NaiveTime::from_hms_opt(0, 0, 0).unwrap() - now.time())
                        .to_std()
                        .unwrap();
                    time::sleep(until_midnight).await;
                }
            } else {
                time::sleep(until_start_std).await;

                self.block_until_unblock(start_end_diff, blocker.clone()).await;
            }
        }            
    }
    async fn block_until_unblock(&self, until_unblock: Duration, blocker: Blocker) {
        blocker.block_vec(&self.apps).unwrap();
        self.lock();

        time::sleep(until_unblock).await;

        blocker.unblock_vec(&self.apps).unwrap();
        self.unlock();
    }
}