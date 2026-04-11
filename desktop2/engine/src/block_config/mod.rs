

pub mod timer;
pub mod time_range;

use std::{
    collections::{
        HashMap,
    },
    sync::{
        Arc,
        Mutex,
        atomic::{
            AtomicBool,
            Ordering,
        },
    }
};

use serde::{
    Serialize,
    Deserialize
};

use tokio::{
    sync::{
        oneshot,
    },
    task,
    time::{
        Duration
    },
};

use platform::{
    App,
    Blocker,
};

use time_range::{
    WeekdayRuleMode,
};

#[derive(Clone, Serialize, Deserialize)]
pub enum BlockRuleKind {
    Timer(Duration),
    TimeRange(WeekdayRuleMode),
}

pub struct BlockRule {
    pub kind: BlockRuleKind,
    pub lock_when_blocked: bool,
    pub blocker: Blocker,
}

impl BlockRule {
    pub fn from_saved(saved: SavedBlockRule) -> Self {
        BlockRule {
            kind: saved.kind,
            lock_when_blocked: saved.lock_when_blocked,
            
        }
    }
}

pub struct Group {
    pub all_apps: Arc<[App]>,
    pub apps_i: Vec<usize>,
    pub block_rule: Option<BlockRule>,
    is_blocked: AtomicBool,
    is_locked: AtomicBool, 
}

impl Group {
    pub fn from_apps_i(apps_i: Vec<usize>, all_apps: Arc<[App]>) -> Self {
        let is_blocked = AtomicBool::new(false);
        let is_locked = AtomicBool::new(false);

        Group { 
            all_apps,
            apps_i,  
            block_rule: None,
            is_blocked, 
            is_locked,
        }        
    }

    pub fn from_app_names(app_names: Vec<String>, all_apps: Arc<[App]>) -> Self {
        let apps_i = app_names_into_idx(app_names, &all_apps);

        Group::from_apps_i(apps_i, all_apps)
    }

    pub fn from_saved(saved: SavedGroup, all_apps: Arc<[App]>) -> Self {
        let apps_i = app_names_into_idx(saved.app_names, &all_apps);

        Group {
            all_apps,
            apps_i,
            
        }
    }

    pub fn apps(&self) -> Vec<&App> {
        self.apps_i
            .iter()
            .map(|i| &self.all_apps[*i])
            .collect()
    }

    pub fn app_names_owned(&self) -> Vec<String> {
        self.apps_i
            .iter()
            .map(|i| self.all_apps[*i].name().clone())
            .collect()
    }

    pub fn is_blocked(&self) -> bool {
        self.is_blocked.load(Ordering::Relaxed)
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked.load(Ordering::Relaxed)
    }

    pub async fn block(&self, block_rule: BlockRule) {
        self.is_blocked.store(true, Ordering::Relaxed);

        let apps = self.apps();
        let blocker = block_rule.blocker;
        let lock_when_blocked = block_rule.lock_when_blocked;

        match block_rule.kind {
            BlockRuleKind::Timer(duration) => self.block_with_timer(duration, apps, blocker, lock_when_blocked).await,
            BlockRuleKind::TimeRange(weekday_rule_mode) => self.block_with_time_range(weekday_rule_mode, apps, blocker, lock_when_blocked).await,
        };               
    }
    pub fn unblock(&self) -> Result<(), &'static str> {
        let is_locked = self.is_locked();

        if is_locked {
            return Err("group is locked")
        }

        self.set_is_blocked(false);
        
        Ok(())
    }
    fn set_is_blocked(&self, value: bool) {
        self.is_blocked.store(value, Ordering::Relaxed);
    }
    fn set_is_locked(&self, value: bool) {
        self.is_locked.store(value, Ordering::Relaxed);
    }
    fn lock(&self) {
        self.is_locked.store(true, Ordering::Relaxed);
    } 
    fn unlock(&self) {
        self.is_locked.store(false, Ordering::Relaxed);
    }

}

pub fn app_names_into_idx(app_names: Vec<String>, all_apps: &[App]) -> Vec<usize> {
    let mut name_index: HashMap<&String, usize> = all_apps
        .iter()
        .enumerate()
        .map(|(i, app)| (app.name(), i))
        .collect();

    app_names
        .into_iter()
        .filter_map(|name| name_index.remove(&name))
        .collect()
}

#[derive(Serialize, Deserialize)]
pub struct SavedBlockRule {
    pub kind: BlockRuleKind,
    pub lock_when_blocked: bool,
}

impl SavedBlockRule {
    pub fn from_block_rule(block_rule: &BlockRule) -> Self {
        Self {
            kind: block_rule.kind.clone(),
            lock_when_blocked: block_rule.lock_when_blocked
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedGroup {
    app_names: Vec<String>,
    block_rule: Option<SavedBlockRule>,
}

impl SavedGroup {
    pub fn from_group(group: &Group) -> Self {
        let app_names = group.app_names_owned();

        let saved_block_rule = group.block_rule
            .as_ref()
            .map(|b_r| {
                SavedBlockRule::from_block_rule(b_r)
            });

        SavedGroup {
            app_names,
            block_rule: saved_block_rule
        }
    }
}