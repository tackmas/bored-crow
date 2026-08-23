mod weekday_set_serde;

use std::ops::{Index, IndexMut};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{
    self, Datelike, DateTime, FixedOffset, Local, NaiveTime, Offset, TimeDelta, Timelike, TimeZone, Weekday, WeekdaySet,
};

use serde::{Deserialize, Serialize};

use tokio::task;
use tokio::time::{self, Duration};

use crate::core::local_clock::LocalClock;
use crate::platform::Blocker;

use super::{CommonBlockInfo, Group, LockWhenBlocked};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: LocalClock,
    pub end: LocalClock,
}

impl TimeRange {
    pub fn min() -> Self {    
        let start = LocalClock::midnight();
        let end = {
            let local_time = NaiveTime::from_hms_opt(0, 1, 0)
                .expect("if this fails this you are trolling");

            LocalClock::from_local_time(local_time)
        };

        Self {
            start,
            end
        }
    }

    pub fn create(start: LocalClock, end: LocalClock) -> Self {
        Self {
            start,
            end
        }
    }

    pub fn duration(&self) -> Duration {
        let mut diff = self.end - self.start;

        if diff < TimeDelta::zero() {
            diff += TimeDelta::hours(24);
        }

        diff.to_std().unwrap()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CustomWeekdays {
    pub monday: CustomWeekday,
    pub tuesday: CustomWeekday,
    pub wednesday: CustomWeekday,
    pub thursday: CustomWeekday,
    pub friday: CustomWeekday,
    pub saturday: CustomWeekday,
    pub sunday: CustomWeekday,
}

impl CustomWeekdays {
    fn set_from_fields(&self) -> WeekdaySet {
        let arr = [
            &self.monday,
            &self.tuesday,
            &self.wednesday,
            &self.thursday,
            &self.friday,
            &self.saturday,
            &self.sunday,
        ];

        arr.into_iter()
            .enumerate()
            .filter(|(_, weekday)| weekday.enabled)
            .map(|(i, _)| Weekday::try_from(i as u8).unwrap())
            .collect()
    }
}

impl Index<u8> for CustomWeekdays {
    type Output = CustomWeekday;

    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => &self.monday,
            1 => &self.tuesday,
            2 => &self.wednesday,
            3 => &self.thursday,
            4 => &self.friday,
            5 => &self.saturday,
            6 => &self.sunday,
            _ => panic!("Index must be between the range 0-6 (inclusive)")
        }
    }
}

impl IndexMut<u8> for CustomWeekdays {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.monday,
            1 => &mut self.tuesday,
            2 => &mut self.wednesday,
            3 => &mut self.thursday,
            4 => &mut self.friday,
            5 => &mut self.saturday,
            6 => &mut self.sunday,
            _ => panic!("Index must be between the range 0-6 (inclusive)")
        }        
    }
}

impl Index<Weekday> for CustomWeekdays {
    type Output = CustomWeekday;

    fn index(&self, index: Weekday) -> &Self::Output {
        match index {
            Weekday::Mon => &self.monday,
            Weekday::Tue => &self.tuesday,
            Weekday::Wed => &self.wednesday,
            Weekday::Thu => &self.thursday,
            Weekday::Fri => &self.friday,
            Weekday::Sat => &self.saturday,
            Weekday::Sun => &self.sunday,
        }        
    }
}

impl IndexMut<Weekday> for CustomWeekdays {
    fn index_mut(&mut self, index: Weekday) -> &mut Self::Output {
        match index {
            Weekday::Mon => &mut self.monday,
            Weekday::Tue => &mut self.tuesday,
            Weekday::Wed => &mut self.wednesday,
            Weekday::Thu => &mut self.thursday,
            Weekday::Fri => &mut self.friday,
            Weekday::Sat => &mut self.saturday,
            Weekday::Sun => &mut self.sunday,
        }          
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomWeekday {
    pub time_range: TimeRange,
    pub enabled: bool,
}

impl CustomWeekday {
    pub fn enabled_time_range(&self) -> Option<&TimeRange> {
        self.enabled.then_some(&self.time_range)
    }
    pub fn enabled_time_range_mut(&mut self) -> Option<&mut TimeRange> {
        self.enabled.then_some(&mut self.time_range)
    }
}

impl Default for CustomWeekday {
    fn default() -> Self {
        Self {
            time_range: TimeRange::min(),
            enabled: false
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UniformWeekdays {
    pub time_range: TimeRange,
    #[serde(with = "weekday_set_serde")]
    pub weekdays: WeekdaySet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WeekdayRuleMode {
    Custom(CustomWeekdays),
    Uniform(UniformWeekdays),
}

impl WeekdayRuleMode {
    fn weekday_set(&self) -> WeekdaySet {
        match self {
            WeekdayRuleMode::Custom(custom_weekday) => custom_weekday.set_from_fields(),
            WeekdayRuleMode::Uniform(uniform_weekdays) => uniform_weekdays.weekdays,
        }
    }
    fn time_range(&self, weekday: Weekday) -> &TimeRange {
        match self {
            WeekdayRuleMode::Custom(custom_weekdays) => {
                custom_weekdays[weekday]
                    .enabled_time_range()
                    .unwrap()
            }
            WeekdayRuleMode::Uniform(uniform_weekdays) => &uniform_weekdays.time_range,
        }
    }
    fn info(&self) -> (LocalClock, TimeDelta, Duration, Duration) {
        let now = chrono::Local::now();
        let now_time = LocalClock::now();
        //println!("{:?}", now_time);

        let now_weekday = now.weekday();

        let start_weekday = self
            .weekday_set()
            .iter(now_weekday)
            .next()
            .expect("Should always be a weekday in the set");

        //println!("{start_weekday}");

        let time_range = self.time_range(start_weekday);

        let until_start = until_start(now_weekday, now_time, start_weekday, time_range.start);

        // println!("Chrono: {until_start:?}");

        let until_start_std = until_start
            .abs()
            .to_std()
            .expect("TimeDelta should always be absolute value");

        //println!("Std: {until_start_std:?}");

        let time_range_duration = time_range.duration();        

        (now_time, until_start, until_start_std, time_range_duration)
    }
    async fn logic(
        &self,
        if_pre_start_time: impl AsyncFnOnce(), 
        if_inside_time_range: impl AsyncFnOnce(), 
        if_post_end_time: impl AsyncFnOnce()
    ) {
        let (
            _now_time,
            until_start, 
            until_start_unsigned, 
            time_range_duration
        ) = self.info();

        let is_post_start_time = until_start.num_seconds().is_negative();

        if is_post_start_time {
            let is_pre_end_time = until_start_unsigned < time_range_duration;

            if is_pre_end_time {
                if_inside_time_range().await
            } else {
                if_post_end_time().await
            }
        } 
        else {
            if_pre_start_time().await
        }
    }
}


impl Group {
    pub(super) async fn block_with_time_range(
        self: Arc<Self>,
        weekday_rule_mode: WeekdayRuleMode,
        lock_when_blocked: LockWhenBlocked,
        cbi: CommonBlockInfo,
    ) {
        // println!("Blocking with Time Range. WeekdayRuleMode: {:?}", weekday_rule_mode);

        let CommonBlockInfo {
            blocker,
            unblock_rx,
        } = cbi;

        let empty_closure = async || {};

        let init_block = async || {
            if let Some(more_lock_config) = *lock_when_blocked {
                self.lock(more_lock_config, &blocker);
            } 

            self.block_apps(&blocker).await;
        };

        weekday_rule_mode.logic(empty_closure, init_block, empty_closure).await;

        task::spawn(async move {
            let run_block = async { loop {
                let (
                    now_time,
                    _until_start, 
                    until_start_unsigned, 
                    time_range_duration
                ) = weekday_rule_mode.info();

                let if_pre_start_time = async || {
                    time::sleep(until_start_unsigned).await;

                    self.block_until_unblock(time_range_duration, &blocker, lock_when_blocked)
                        .await;  
                };

                let if_inside_time_range = async || {
                    let until_end = time_range_duration - until_start_unsigned;

                    self.block_until_unblock(until_end, &blocker, lock_when_blocked).await;
                };

                let if_post_end_time = async || {
                    let until_midnight = until_midnight(now_time);

                    println!("{until_midnight:?}");

                    time::sleep(until_midnight).await;
                };

                weekday_rule_mode.logic(if_pre_start_time, if_inside_time_range, if_post_end_time).await;
            }};

            tokio::select! {
                result = unblock_rx => {
                    result.unwrap();

                    self.unblock_apps(&blocker).await;
                },
                _ = run_block => {}
            }
        });
    }

    async fn block_until_unblock<'a>(
        &self,
        until_unblock: Duration,
        blocker: &Blocker,
        lock_when_blocked: LockWhenBlocked,
    ) {
        if let Some(more_lock_config) = *lock_when_blocked {
            self.lock(more_lock_config, blocker);
        } 
        self.block_apps(blocker).await;

        time::sleep(until_unblock).await;

        if let Some(more_lock_config) = *lock_when_blocked {
            self.unlock(more_lock_config, blocker);
        } 

        self.block_apps(blocker).await;
    }
}

fn until_start(
    now_weekday: Weekday,
    now_time: LocalClock,
    start_weekday: Weekday,
    start_clock: LocalClock,
) -> TimeDelta {
    let until_start_day = {
        let day_diff = start_weekday.days_since(now_weekday);

        TimeDelta::days(day_diff as i64)
    };

    let until_start_clock = start_clock - now_time;

    until_start_day + until_start_clock
}

fn until_midnight(now_time: LocalClock) -> Duration {
    let day_in_seconds = 24 * 60 * 60;
    let now_time = now_time.time();
    let remaining_secs_ceiling =
        TimeDelta::seconds((day_in_seconds - now_time.num_seconds_from_midnight()) as i64);
    let remaining_secs = remaining_secs_ceiling - TimeDelta::nanoseconds(now_time.nanosecond() as i64);

    remaining_secs.to_std().unwrap()
}
