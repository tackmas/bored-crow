pub mod custom;
pub mod uniform;

use std::fmt::Debug;
use std::sync::Arc;

use chrono::{
    self, 
    Datelike, 
    FixedOffset, 
    Local, 
    NaiveTime, 
    TimeDelta, 
    Timelike, 
    Weekday, 
    WeekdaySet,
};

use serde::{Deserialize, Serialize};

use tokio::task;
use tokio::time::{self, Duration};

use crate::platform::Blocker;

pub use self::custom::{CustomWeek, TimeRangesOnWeek};
pub use self::uniform::UniformWeekdays;

use super::{CommonBlockInfo, Group, LockWhenBlocked};
use super::LockConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

impl TimeRange {
    pub fn min() -> Self {    
        let start = NaiveTime::MIN;
        let end = NaiveTime::from_hms_opt(0, 1, 0)
            .expect("if this fails this you are trolling");

        Self {
            start,
            end
        }
    }

    pub fn create(start: NaiveTime, end: NaiveTime) -> Self {
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

impl Default for TimeRange {
    fn default() -> Self {
        Self::min()
    }
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WeekSchedule<T: WeekScheduleT> {
    pub inner: T,
    #[serde(with = "fixed_offset_serde")]
    pub local_minus_utc: FixedOffset
}

impl<T: WeekScheduleT> WeekSchedule<T> {
    async fn logic(
        &self,
        if_pre_start_time: impl AsyncFnOnce(Duration), 
        if_inside_time_range: impl AsyncFnOnce(), 
        if_post_end_time: impl AsyncFnOnce()
    ) {
        let (
            _now_time,
            until_start, 
            until_start_unsigned, 
            time_range_duration
        ) = get_common_info(self);

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
            if_pre_start_time(until_start_unsigned).await
        }
    }
}


impl Group {
    pub(super) async fn block_with_time_range<T>(
        self: Arc<Self>,
        week_schedule: WeekSchedule<T>,
        lock_config: LockConfig,
        cbi: CommonBlockInfo,
    ) 
    where 
        T: WeekScheduleT
    {
        let CommonBlockInfo {
            blocker,
            unblock_rx,
        } = cbi;

        let lock_when_blocked = self.lock_when_blocked(lock_config, &blocker);

        let empty_closure1 = async || {};
        let empty_closure2 = async |_| {};

        let init_block = async || {
            if let Some(more_lock_config) = *lock_when_blocked {
                self.lock(more_lock_config, &blocker);
            } 

            self.block_apps(&blocker).await;
        };

        week_schedule.logic(empty_closure2, init_block, empty_closure1).await;

        task::spawn(async move {
            let run_block = async { loop {
                let (
                    now_time,
                    _until_start, 
                    until_start_unsigned, 
                    time_range_duration
                ) = get_common_info(&week_schedule);

                let if_pre_start_time = async |until_start_unsigned: Duration| {
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

                    time::sleep(until_midnight).await;
                };

                week_schedule.logic(if_pre_start_time, if_inside_time_range, if_post_end_time).await;
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

    async fn block_until_unblock(
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

fn get_common_info<T>(week_schedule: &WeekSchedule<T>) -> (NaiveTime, TimeDelta, Duration, Duration) 
where 
    T: WeekScheduleT
{
    let now = Local::now().with_timezone(&week_schedule.local_minus_utc);
    let now_time = now.time();
    let now_weekday = now.weekday();
    let week_schedule = &week_schedule.inner;

    let start_weekday = week_schedule
        .enabled_weekdays()
        .iter(now_weekday)
        .next()
        .expect("Should always be a weekday in the set");
    let time_range = week_schedule.time_range_on_weekday(start_weekday).unwrap();

    let until_start = until_start(now_weekday, now_time, start_weekday, time_range.start);
    println!("Until Start: {until_start:?}");

    let until_start_std = until_start
        .abs()
        .to_std()
        .expect("TimeDelta should always be absolute value");

    let time_range_duration = time_range.duration();        

    (now_time, until_start, until_start_std, time_range_duration)
}

fn until_start(
    now_weekday: Weekday,
    now_clock: NaiveTime,
    start_weekday: Weekday,
    start_clock: NaiveTime,
) -> TimeDelta 
{
    let until_start_day = {
        let day_diff = start_weekday.days_since(now_weekday);

        TimeDelta::days(day_diff as i64)
    };
    println!("Start Clock: {:?}", start_clock);
    println!("Now Clock: {:?}", now_clock);

    let until_start_clock = start_clock - now_clock;

    until_start_day + until_start_clock    
}

fn until_midnight(now_time: NaiveTime) -> Duration {
    let day_in_seconds = 24 * 60 * 60;
    let remaining_secs_ceiling =
        TimeDelta::seconds((day_in_seconds - now_time.num_seconds_from_midnight()) as i64);
    let remaining_secs = remaining_secs_ceiling - TimeDelta::nanoseconds(now_time.nanosecond() as i64);

    remaining_secs.to_std().unwrap()
}

pub trait WeekScheduleT:
    Clone
    + Debug
    + Send
    + Sync
    + 'static
{
    fn enabled_weekdays(&self) -> WeekdaySet;
    fn time_range_on_weekday(&self, weekday: Weekday) -> Option<&TimeRange>;
}

mod fixed_offset_serde {
    use serde::{Deserializer, Deserialize, Serializer, Serialize};

    use super::FixedOffset;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<FixedOffset, D::Error>
    where 
        D: Deserializer<'de>
    {
        let offset_i32 = i32::deserialize(deserializer)?;
        let fixed_offset = FixedOffset::east_opt(offset_i32).unwrap();

        Ok(fixed_offset)
    }

    pub fn serialize<S>(fixed_offset: &FixedOffset, serializer: S) -> Result<S::Ok, S::Error>
    where 
        S: Serializer 
    {
        let offset_i32 = fixed_offset.local_minus_utc();
        offset_i32.serialize(serializer)
    }
}


mod weekday_set_serde {
    use chrono::{Weekday, WeekdaySet};

    use serde::{Deserialize, Deserializer, Serializer, Serialize};

    pub fn serialize<S>(set: &WeekdaySet, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut set_u8 = 0u8;

        for weekday in set.iter(Weekday::Mon) {
            set_u8 |= set_u8_with_weekday(weekday);
        }

        set_u8.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<WeekdaySet, D::Error>
    where
        D: Deserializer<'de>,
    {
        let set_u8 = u8::deserialize(deserializer)?;
        let mut set = WeekdaySet::ALL;

        for weekday in set.iter(Weekday::Mon) {
            if set_u8 & set_u8_with_weekday(weekday) == 0 {
                set.remove(weekday);
            }
        }

        Ok(set)
    }

    fn set_u8_with_weekday(weekday: Weekday) -> u8 {
        1u8 << weekday as u8
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn until_start_works() {
        let clock = |h, m, s| NaiveTime::from_hms_opt(h, m, s).unwrap();

        {
            let now_clock = clock(1, 0, 0);
            let now_weekday = Weekday::Mon;
            let start_clock = clock(3, 0, 0);
            let start_weekday = Weekday::Mon;    

            let until_start = until_start(now_weekday, now_clock, start_weekday, start_clock); 

            assert!(until_start == TimeDelta::hours(2), "{until_start:?}");  
        }

        {
            let now_clock = clock(23, 0, 0);
            let now_weekday = Weekday::Mon;
            let start_clock = clock(1, 0, 0);
            let start_weekday = Weekday::Mon;   
            let until_start = until_start(now_weekday, now_clock, start_weekday, start_clock); 

            assert!(
                until_start == TimeDelta::hours(-22), "{until_start:?}"
            );
        }

        {
            let now_clock = clock(1, 0, 0);
            let now_weekday = Weekday::Mon;
            let start_clock = clock(3, 0, 0);
            let start_weekday = Weekday::Tue;

            let until_start = until_start(now_weekday, now_clock, start_weekday, start_clock);

            assert!(until_start == TimeDelta::hours(26), "{until_start:?}");
        }

        {
            let now_clock = clock(23, 0, 0);
            let now_weekday = Weekday::Mon;
            let start_clock = clock(1, 0, 0);
            let start_weekday = Weekday::Tue;

            let until_start = until_start(now_weekday, now_clock, start_weekday, start_clock);

            assert!(until_start == TimeDelta::hours(2), "{until_start:?}");
        }

    }
}



/*


#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Mode {
    Custom(CustomWeek),
    Uniform(UniformWeekdays)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WeekSchedule {
    pub mode: Mode,
    #[serde(with = "fixed_offset_serde")]
    pub local_minus_utc: FixedOffset
}

impl WeekSchedule {

    // All LocalClock must have the same offset, so we can simply choose whichever field to get the offset
    fn offset(&self) -> FixedOffset {
        self.local_minus_utc
    }
    fn weekday_set(&self) -> WeekdaySet {
        match &self.mode {
            Mode::Custom(custom_weekdays) => custom_weekdays.enabled_weekdays(),
            Mode::Uniform(uniform_weekdays) => uniform_weekdays.weekdays,
        }
    }
    fn time_range(&self, weekday: Weekday) -> &TimeRange {
        match &self.mode {
            Mode::Custom(custom_weekdays) => {
                custom_weekdays[weekday]
                    .enabled_time_range()
                    .unwrap()
            }
            Mode::Uniform(uniform_weekdays) => &uniform_weekdays.time_range,
        }
    }
    fn info(&self) -> (NaiveTime, TimeDelta, Duration, Duration) {
        let now = Local::now().with_timezone(&self.offset());
        let now_time = now.time();
        let now_weekday = now.weekday();

        let start_weekday = self
            .weekday_set()
            .iter(now_weekday)
            .next()
            .expect("Should always be a weekday in the set");
        let time_range = self.time_range(start_weekday);

        let until_start = until_start(now_weekday, now_time, start_weekday, time_range.start);
        println!("Until Start: {until_start:?}");

        let until_start_std = until_start
            .abs()
            .to_std()
            .expect("TimeDelta should always be absolute value");

        let time_range_duration = time_range.duration();        

        (now_time, until_start, until_start_std, time_range_duration)
    }
    async fn logic(
        &self,
        if_pre_start_time: impl AsyncFnOnce(Duration), 
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
            if_pre_start_time(until_start_unsigned).await
        }
    }
}
 */