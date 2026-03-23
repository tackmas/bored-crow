mod timer;
mod time_range;

use std::sync::{
    Arc,
    atomic::{
        AtomicBool,
        Ordering,
    },
};

use tokio::{
    task,
    time::{
        Duration
    },
};


use std::{
    fmt::{
        self,
        Display,
    }
};

use crate::platform::{
    App,
    Blocker,
};

use time_range::{
    WeekdayRuleMode,
};

#[derive(Clone, Copy)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
}

impl Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TimeUnit::Seconds => "Seconds",
            TimeUnit::Minutes => "Minutes",
            TimeUnit::Hours => "Hours",
            TimeUnit::Days => "Days",
            TimeUnit::Weeks => "Weeks",
        };

        write!(f, "{s}")
    }
}

pub struct Time {
    pub value: u64,
    pub time_unit: TimeUnit
}

impl Time {
    pub fn duration_from_time_unit(self) -> Duration {
        let value = self.value;

        match self.time_unit {
            TimeUnit::Seconds => Duration::from_secs(value),
            TimeUnit::Minutes => Duration::from_mins(value),
            TimeUnit::Hours => Duration::from_hours(value),
            TimeUnit::Days => Duration::from_hours(value * 24),
            TimeUnit::Weeks => Duration::from_hours(value * 24 * 7),
        }
    }
}

pub enum BlockRuleKind {
    Timer(Duration),
    TimeRange(WeekdayRuleMode),
}

pub struct BlockRule {
    pub kind: BlockRuleKind,
    pub lock_when_blocked: bool,
    pub blocker: Blocker,
}

pub struct Group {
    pub apps: Vec<App>,
    is_blocked: AtomicBool,
    is_locked: Option<AtomicBool>, 
}

impl Group {
    pub fn new_with(apps: Vec<App>) -> Self {
        let is_blocked = AtomicBool::new(false);
        let is_locked = None;

        Group { 
            apps,  
            is_blocked, 
            is_locked,
        }
    }

    pub async fn block(&self, block_rule: BlockRule) {
        self.is_blocked.store(true, Ordering::Relaxed);

        let blocker = block_rule.blocker;

        use BlockRuleKind::*;

        match block_rule.kind {
            Timer(duration) => self.block_with_timer(duration, blocker).await,
            TimeRange(weekday_rule_mode) => self.block_with_time_range(weekday_rule_mode, blocker).await,
        };               
    }
    pub fn unblock(&self) -> Result<(), &'static str> {
        if let Some(is_locked) = &self.is_locked {
            let is_locked = is_locked.load(Ordering::Relaxed);

            if is_locked {
                return Err("group is locked")
            }
        }

        self.is_blocked.store(false, Ordering::Relaxed);
        
        Ok(())
    }
    fn lock(&self) {
        if let Some(is_locked) = &self.is_locked {
            is_locked.store(true, Ordering::Relaxed);
        }
    } 
    fn unlock(&self) {
        if let Some(is_locked) = &self.is_locked {
            is_locked.store(false, Ordering::Relaxed);
        }
    }
}
