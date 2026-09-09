mod timer;
mod time_range;

use std::sync::{Arc};

use serde::{Deserialize, Serialize};

use tokio::sync::oneshot;
use tokio::task;

use crate::{decrease_prohibit_uninstall_count, increase_prohibit_uninstall_count};
use crate::platform::{App, Blocker};

use super::Group;
use super::id::{AsId, Id};

pub use self::timer::Timer;
pub use self::time_range::{CustomWeek, TimeRange, TimeRangesOnWeek, UniformWeekdays, WeekSchedule, WeekScheduleT};


#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BlockRuleKind {
    // Timer
    Timer(Timer, LockWhenBlocked),
    // Time range
    CustomWeekSchedule(WeekSchedule<CustomWeek>, LockConfig),
    UniformWeekSchedule(WeekSchedule<UniformWeekdays>, LockConfig)
}

impl BlockRuleKind {
    pub(super) async fn block_group(
        self, group: Arc<Group>, blocker: Blocker, unblocker_rx: oneshot::Receiver<()>
    ) {
        match self {
            BlockRuleKind::Timer(timer, lock_when_blocked) => {
                timer.block_group(group, lock_when_blocked, blocker, unblocker_rx).await;
            },
            BlockRuleKind::CustomWeekSchedule(custom_week_schedule, lock_config) => {
                custom_week_schedule.block_with_time_range(group, lock_config, blocker, unblocker_rx).await
            }
            BlockRuleKind::UniformWeekSchedule(uniform_week_schedule, lock_config) => {
                uniform_week_schedule.block_with_time_range(group, lock_config, blocker, unblocker_rx).await;
            }
        };
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockConfig { 
    pub id: Id,
    pub kind: BlockRuleKind,
}

impl BlockConfig {
    pub fn from_parts(id: Id, kind: BlockRuleKind) -> Self {
        Self { id, kind }
    }
    pub fn from_saved(kind: BlockRuleKind, id: Id) -> Self {
        BlockConfig {
            id,
            kind
        }
    }
}

impl AsId for BlockConfig {
    fn as_id(&self) -> Id {
        self.id
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LockConfig {
    NoLock,
    LockWhenBlocked(MoreLockConfig),
    Timer(Timer, MoreLockConfig),
} 

impl LockConfig {
    pub(super) fn into_lock_when_blocked(self, group: &Arc<Group>, blocker: &Blocker) -> LockWhenBlocked {
        match self {
            LockConfig::NoLock => None.into(),
            LockConfig::LockWhenBlocked(more_lock_config) => Some(more_lock_config).into(),
            LockConfig::Timer(timer, more_lock_config) => {
                let (blocker, group) = (blocker.clone(), group.clone());

                task::spawn(async move {
                    timer.lock_timer(&group, more_lock_config, &blocker).await;
                });

                None.into()
            }
        }
    
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct MoreLockConfig {
    pub block_task_manager: bool,
    pub prohibit_uninstall: bool,
}

impl MoreLockConfig {
    pub fn enable(self, blocker: &Blocker) {
        let Self { block_task_manager, prohibit_uninstall } = self;

        if block_task_manager {
            todo!()
        };

        if prohibit_uninstall {
            increase_prohibit_uninstall_count();
        }

        println!("Enabled MoreLockConfig")
    }
    pub fn disable(self, blocker: &Blocker) {
        let Self { block_task_manager, prohibit_uninstall } = self;

        if block_task_manager {
            todo!()
        };

        if prohibit_uninstall {
            decrease_prohibit_uninstall_count();
        }
        println!("Disabled MoreLockConfig")
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct LockWhenBlocked {
    flag: Option<MoreLockConfig>
}

impl LockWhenBlocked {
    pub fn from_lock_config(lock_config: &LockConfig) -> Self {
        match lock_config {
            LockConfig::NoLock => LockWhenBlocked::from(None),
            LockConfig::LockWhenBlocked(more_lock_config) => {
                LockWhenBlocked::from(Some(*more_lock_config))
            },
            LockConfig::Timer(..) => {
                panic!("LockConfig cannot be LockConfig::Timer in this method")
            }
        }
    }
}

impl From<Option<MoreLockConfig>> for LockWhenBlocked {
    fn from(value: Option<MoreLockConfig>) -> Self {
        LockWhenBlocked { flag: value }
    }
}