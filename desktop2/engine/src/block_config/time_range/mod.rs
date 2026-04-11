mod weekday_set_serde;

use std::{
    fmt,
    sync::{
        atomic::{
            AtomicBool
        }
    }
};


use chrono::{
    self,
    NaiveTime,
    Datelike,
    DateTime,
    Local,
    TimeDelta,
    Timelike,
    Weekday,
    WeekdaySet,
};

use serde::{
    Serialize,
    Deserialize,
};

use tokio::time::{
    self,
    Duration
};

use platform::{
    App,
    Blocker,
};


use super::{
    Group,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: NaiveTime,
    pub end: NaiveTime
}

impl TimeRange {
    pub const fn min() -> Self {
        TimeRange {
            start: NaiveTime::MIN,
            end: NaiveTime::from_hms_opt(0, 1, 0).expect("if this fails this you are trolling")
        }
    }

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

impl From<(NaiveTime, NaiveTime)> for TimeRange {
    fn from(from: (NaiveTime, NaiveTime)) -> Self {
        let (start, end) = from;

        TimeRange {
            start,
            end,
        }
    }
} 

#[derive(Clone, Serialize, Deserialize)]
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
    pub fn weekday_time_range_mut(&mut self, weekday: Weekday) -> &mut Option<TimeRange> {
        match weekday {
            Weekday::Mon => &mut self.monday,
            Weekday::Tue => &mut self.tuesday,
            Weekday::Wed => &mut self.wednesday,
            Weekday::Thu => &mut self.thursday,
            Weekday::Fri => &mut self.friday,
            Weekday::Sat => &mut self.saturday,
            Weekday::Sun => &mut self.sunday,
        }     
    }

    pub fn weekday_time_range(&self, weekday: Weekday) -> Option<&TimeRange> {
        match weekday {
            Weekday::Mon => &self.monday,
            Weekday::Tue => &self.tuesday,
            Weekday::Wed => &self.wednesday,
            Weekday::Thu => &self.thursday,
            Weekday::Fri => &self.friday,
            Weekday::Sat => &self.saturday,
            Weekday::Sun => &self.sunday,
        }
        .as_ref()
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

#[derive(Clone, Serialize, Deserialize)]
pub struct UniformWeekdays {
    pub time_range: TimeRange,
    #[serde(with = "weekday_set_serde")]
    pub weekdays: WeekdaySet,
}   

#[derive(Clone, Serialize, Deserialize)]
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
            WeekdayRuleMode::Custom(custom_weekday) => custom_weekday
                .weekday_time_range(weekday)
                .as_ref()
                .unwrap(),
            WeekdayRuleMode::Uniform(uniform_weekdays) => &uniform_weekdays.time_range
        }
    }
}


impl Group {
    pub async fn block_with_time_range(
        &self, 
        weekday_rule_mode: WeekdayRuleMode, 
        apps: Vec<&App>,
        blocker: Blocker,
        lock_when_blocked: bool
    ) {
        loop {
            let now = chrono::Local::now();
            let now_time = now.time();
            let now_weekday = now.weekday();

            let set = weekday_rule_mode.weekday_set();

            let start_weekday = set
                .iter(now_weekday)
                .next()
                .expect("Should always be a weekday in the set");

            let time_range = weekday_rule_mode.time_range(start_weekday);

            let until_start = until_start(start_weekday, time_range.start, now_weekday, now_time);

            let until_start_std = until_start
                .abs()
                .to_std()
                .unwrap();

            let start_end_diff = time_range.unsigned_diff();

            if until_start.num_seconds() < 0 {
                if until_start_std < start_end_diff {
                    let until_end = start_end_diff - until_start_std;

                    if self.is_blocked() {
                        self.block_until_unblock(until_end, &apps, blocker.clone(), lock_when_blocked).await;
                    } else {
                        break;
                    }
                } else {
                    let until_midnight = until_midnight(now_time);

                    println!("{until_midnight:?}");

                    time::sleep(until_midnight).await;
                }
            } else {
                time::sleep(until_start_std).await;

                if self.is_blocked() {
                    self.block_until_unblock(start_end_diff, &apps, blocker.clone(), lock_when_blocked).await;
                } else {
                    break;
                }
            }
        }            
    }

    async fn block_until_unblock(
        &self, 
        until_unblock: Duration, 
        apps: &Vec<&App>,
        blocker: Blocker, 
        lock_when_blocked: bool
    ) {
        self.set_is_locked(lock_when_blocked);
        blocker.block_vec(&apps).await.unwrap();

        time::sleep(until_unblock).await;

        self.unlock();
        blocker.unblock_vec(&apps).await.unwrap();
    }
}

fn until_start(
    start_weekday: Weekday, 
    start_clock: NaiveTime, 
    now_weekday: Weekday, 
    now_time: NaiveTime
) -> TimeDelta 
{
    let until_start_day = {
        let day_diff = start_weekday.days_since(now_weekday);  

        TimeDelta::days(day_diff as i64)
    };

    let until_start_clock = start_clock.signed_duration_since(now_time);
    
    until_start_day + until_start_clock
}

fn until_midnight(time: NaiveTime) -> Duration {
    let day_in_seconds = 24 * 60 * 60;
    let remaining_secs_ceiling = TimeDelta::seconds(
        (day_in_seconds - time.num_seconds_from_midnight()) as i64
    );
    let remaining = remaining_secs_ceiling - TimeDelta::nanoseconds(time.nanosecond() as i64);

    remaining
        .to_std()
        .unwrap()
}