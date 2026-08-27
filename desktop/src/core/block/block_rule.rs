use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::{decrease_prohibit_uninstall_count, increase_prohibit_uninstall_count};
use crate::platform::{App, Blocker};

use super::id::{AsId, Id};
use super::timer::Timer;
use super::time_range::{CustomWeek, UniformWeekdays, WeekSchedule};


#[derive(Clone, Serialize, Deserialize)]
pub enum BlockRuleKind {
    // Timer
    Timer { timer: Timer, lock_when_blocked: LockWhenBlocked },
    // Time range
    CustomWeekSchedule(WeekSchedule<CustomWeek>, LockConfig),
    UniformWeekSchedule(WeekSchedule<UniformWeekdays>, LockConfig)
}


#[derive(Clone)]
pub struct BlockConfig{
    pub id: Id,
    pub kind: BlockRuleKind,
}

impl BlockConfig {
    pub fn from_parts(id: Id, kind: BlockRuleKind) -> Self {
        Self { id, kind }
    }
    pub fn from_saved(saved: SavedBlockConfig, id: Id) -> Self {
        BlockConfig {
            id,
            kind: saved.kind,
        }
    }
}

impl AsId for BlockConfig {
    fn as_id(&self) -> Id {
        self.id
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LockConfig {
    NoLock,
    LockWhenBlocked(MoreLockConfig),
    Timer(Timer, MoreLockConfig),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct LockWhenBlocked(Option<MoreLockConfig>);

impl LockWhenBlocked {
    pub fn from_lock_config(lock_config: &LockConfig) -> Self {
        match lock_config {
            LockConfig::NoLock => LockWhenBlocked(None),
            LockConfig::LockWhenBlocked(more_lock_config) => {
                LockWhenBlocked(Some(*more_lock_config))
            },
            LockConfig::Timer(..) => {
                panic!("LockConfig cannot be LockConfig::Timer in this method")
            }
        }
    }
}

impl From<Option<MoreLockConfig>> for LockWhenBlocked {
    fn from(value: Option<MoreLockConfig>) -> Self {
        LockWhenBlocked(value)
    }
}

impl Deref for LockWhenBlocked {
    type Target = Option<MoreLockConfig>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedBlockConfig {
    pub kind: BlockRuleKind,
}

impl SavedBlockConfig {
    pub fn from_block_rule(block_rule: &BlockConfig) -> Self {
        Self {
            kind: block_rule.kind.clone(),
        }
    }
}

