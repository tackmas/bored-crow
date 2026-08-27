use std::ops::{Add, Sub};

use chrono::{FixedOffset, Local, NaiveTime, TimeDelta};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Clock {
    utc_plus_offset: NaiveTime,
    #[serde(with = "fixed_offset_serde")]
    offset: FixedOffset
}

impl Clock {
    pub fn local_now() -> Self {
        let now = Local::now();
        let offset = *now.offset();
        let local_time = now.time();

        Self {
            utc_plus_offset: local_time,
            offset
        }
    }

    pub fn local_midnight() -> Self {
        let offset = *Local::now().offset();
        let local_midnight = NaiveTime::MIN;

        Self {
            utc_plus_offset: local_midnight,
            offset
        }
    }

    pub fn from_local_time(local_time: NaiveTime) -> Self {
        let offset = *Local::now().offset();

        Self {
            utc_plus_offset: local_time,
            offset
        }
    }
    pub fn from_local_hms(h: u32, m: u32, s: u32) -> Self {
        let local_time = NaiveTime::from_hms_opt(h, m, s).expect("Invalid arguments");
        Self::from_local_time(local_time)
    }

    pub fn as_naive_time(&self) -> NaiveTime {
        self.utc_plus_offset
    }
    #[cfg(any(debug_assertions, test))]
    pub fn with_offset(&mut self, offset: FixedOffset) {
        self.offset = offset;
    }

    pub fn with_local_offset(&mut self, other: &Clock) {
        self.offset = other.offset;
    }
    pub fn offset(&self) -> FixedOffset {
        self.offset
    }
}

impl Sub for Clock {
    type Output = TimeDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(
            self.offset == rhs.offset, 
            "Both LocalClock must have the same offset for it to be correct. Otherwise use NaiveTime"
        );

        self.utc_plus_offset - rhs.utc_plus_offset
    }
}



fn create_naive_time(h: u32, m: u32, s: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, s).unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naive_time_minus_time_delta() {
        let naive_time = create_naive_time(3, 0, 0);
        let time_delta = TimeDelta::hours(2);
        let result = naive_time - time_delta;
        assert!(result == create_naive_time(1, 0, 0), "{result:?}");

        let naive_time = create_naive_time(1, 0, 0);
        let time_delta = TimeDelta::hours(2);
        let result = naive_time - time_delta;
        assert!(result == create_naive_time(23, 0, 0), "{result:?}");

        let naive_time = create_naive_time(4, 0, 0);
        let time_delta = TimeDelta::hours(27);
        let result = naive_time - time_delta;
        assert!(result == create_naive_time(1, 0, 0), "{result:?}");
    }

    #[test]
    fn local_clock_minus_local_clock() {
        let local_clock = Clock::from_local_hms(3, 0, 0);
        let rhs = Clock::from_local_hms(1, 0, 0);
        let result = local_clock - rhs;
        assert!(result == TimeDelta::hours(2), "{result:?}");

        let local_clock = Clock::from_local_hms(1, 0, 0);
        let rhs = Clock::from_local_hms(3, 0, 0);
        let result = local_clock - rhs;
        assert!(result == TimeDelta::hours(-2), "{result:?}");
        
        let local_clock = Clock::from_local_hms(23, 0, 0);
        let rhs = Clock::from_local_hms(1, 0, 0);
        let result = local_clock - rhs;
        assert!(result == TimeDelta::hours(22), "{result:?}");
    }
}