use std::ops::{Add, Sub};

use chrono::{FixedOffset, Local, NaiveTime, TimeDelta};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug)]
pub struct LocalClock {
    utc_time: NaiveTime,
    offset: FixedOffset
}

impl LocalClock {
    pub fn now() -> Self {
        let now = Local::now();
        let offset = *now.offset();
        let utc_time = now.to_utc().time();

        Self {
            utc_time,
            offset
        }
    }

    pub fn midnight() -> Self {
        let offset = *Local::now().offset();
        let utc_time = NaiveTime::MIN - offset;

        Self {
            utc_time,
            offset
        }
    }

    pub fn from_local_time(local_time: NaiveTime) -> Self {
        let offset = *Local::now().offset();
        let utc_time = local_time - offset;

        Self {
            utc_time,
            offset
        }
    }

    pub fn time(&self) -> NaiveTime {
        self.utc_time + self.offset
    }
}

impl Sub for LocalClock {
    type Output = TimeDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(
            self.offset == rhs.offset, 
            "Both LocalClock must have the same offset for it to be correct. Otherwise use NaiveTime"
        );

        self.utc_time - rhs.utc_time
    }
}

impl<'de> Deserialize<'de> for LocalClock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let utc_time = NaiveTime::deserialize(deserializer)?;
        let offset = *Local::now().offset();

        Ok(Self {
            utc_time,
            offset
        })

    }
}

impl Serialize for LocalClock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        self.utc_time.serialize(serializer)
    }
}