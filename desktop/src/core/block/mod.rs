pub mod block_rule;
pub mod id;
pub mod time_range;
pub mod timer;

pub use block_rule::*;

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use serde::{Deserialize, Serialize};

use tokio::{
    sync::oneshot,
    task::{self, JoinHandle},
    time::Duration,
};

use crate::platform::{App, Blocker};
use self::id::{AsId, Id};

use timer::Timer;

use time_range::WeekScheduleT;

pub struct Unblocker {
    unblock_tx: oneshot::Sender<()>,
}

impl Unblocker {
    fn new(unblock_tx: oneshot::Sender<()>) -> Self {
        Self { unblock_tx }
    }
    fn unblock(self) {
        self.unblock_tx.send(()).unwrap()
    }
}

struct CommonBlockInfo {
    blocker: Blocker,
    unblock_rx: oneshot::Receiver<()>,
}

impl CommonBlockInfo {
    fn create(blocker: Blocker, unblock_rx: oneshot::Receiver<()>) -> Self {
        Self {
            blocker,
            unblock_rx,
        }
    }
}

pub struct Group {
    pub id: Id,
    pub all_apps: Arc<[App]>,
    pub apps_i: Vec<usize>,
    unblocker_tx: Mutex<Option<Unblocker>>,
    is_locked: AtomicBool,
}

impl Group {
    pub fn from_apps_i(apps_i: Vec<usize>, all_apps: Arc<[App]>, id: Id) -> Self {
        let unblocker_tx = Mutex::new(None);
        let is_locked = AtomicBool::new(false);

        Group {
            id,
            all_apps,
            apps_i,
            unblocker_tx,
            is_locked,
        }
    }

    pub fn from_app_names(app_names: Vec<String>, all_apps: Arc<[App]>, id: Id) -> Self {
        let apps_i = app_names_into_idx(app_names, &all_apps);

        Group::from_apps_i(apps_i, all_apps, id)
    }

    pub async fn from_saved(saved: SavedGroup, all_apps: Arc<[App]>, blocker: &Blocker) -> Arc<Self> {
        let apps_i = app_names_into_idx(saved.app_names, &all_apps);

        let id = saved.id.into();

        let block_rule_opt = saved
            .block_rule_opt
            .map(|saved_block_rule| BlockConfig::from_saved(saved_block_rule, id));

        let group = Arc::new(Group {
            id,
            all_apps,
            apps_i,
            unblocker_tx: Mutex::new(None),
            is_locked: AtomicBool::new(false),
        });

        if let Some(block_rule) = block_rule_opt {
            let (group, blocker) = (group.clone(), blocker.clone());
            group.block(block_rule, blocker).await;
        }
        println!("Loaded Group (Id: {})", group.as_id());

        group
    }

    async fn block_apps(&self, blocker: &Blocker) {
        for &i in &self.apps_i {
            blocker.block(&self.all_apps[i]).await;
        }
    }

    async fn unblock_apps(&self, blocker: &Blocker) {
        for &i in &self.apps_i {
            blocker.unblock(&self.all_apps[i]).await;
        }
    }

    pub fn app_names_owned(&self) -> Vec<String> {
        self.apps_i
            .iter()
            .map(|i| self.all_apps[*i].name().clone())
            .collect()
    }

    pub fn is_blocked(&self) -> bool {
        let is_blocked = self.unblocker_tx.lock().unwrap().is_some();

        println!("Is blocked: '{is_blocked}'");

        is_blocked
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked.load(Ordering::Acquire)
    }

    pub async fn block(self: Arc<Self>, block_config: BlockConfig, blocker: Blocker) {
        let (tx, rx) = oneshot::channel::<()>();
        {
            let mut guard = self.unblocker_tx.lock().unwrap();

            if guard.is_some() {
                eprintln!("Already blocked");

                return;
            }

            *guard = Some(Unblocker::new(tx));

            println!("Set 'Unblocker' to 'Some'");
        };
        
        let cbi = CommonBlockInfo::create(blocker, rx);

        println!("Blocking Group (Id: {})", self.as_id());

        match block_config.kind {
            BlockRuleKind::Timer { timer, lock_when_blocked } => {
                self.block_with_timer(timer, lock_when_blocked, cbi).await;
            },
            BlockRuleKind::CustomWeekSchedule(week_schedule, lock_config) => {
                self.block_with_time_range(week_schedule, lock_config, cbi).await
            }
            BlockRuleKind::UniformWeekSchedule(week_schedule, lock_config) => {
                self.block_with_time_range(week_schedule, lock_config, cbi).await;
            }
        };
    }
    pub fn unblock(&self) -> Result<(), &'static str> {
        let is_locked = self.is_locked();

        if is_locked {
            eprintln!("Group is locked");
            return Err("group is locked");
        }

        let Some(unblocker_tx) = self.take_unblocker() else {
            let err_msg = "Already is not blocked (unblock)";
            eprintln!("{err_msg}");
            return Err(err_msg);
        };
        unblocker_tx.unblock();

        println!("Unblocked Group (Id: {})", self.as_id());

        Ok(())
    }
    pub fn force_unblock(&self) {
        let Some(unblocker_tx) = self.take_unblocker() else {
            eprintln!("Already is not blocked (force_unblock)");
            return;
        };
        unblocker_tx.unblock();

        println!("Forcibly unblocked Group (Id: {})", self.as_id());
    }
    fn clear_unblocker(&self) {
        let _ = self.take_unblocker();

        println!("Cleared Unblocker in Group (Id: {})", self.as_id());
    }
    fn take_unblocker(&self) -> Option<Unblocker> {
        let unblocker = self.unblocker_tx.lock().unwrap().take();

        println!("Took Unblocker in Group (Id: {})", self.as_id());

        unblocker
    }
    fn lock(&self, more_lock_config: MoreLockConfig, blocker: &Blocker) {
        self.is_locked.store(true, Ordering::Release);

        more_lock_config.enable(blocker);

        println!("Locked Group (Id: {})", self.as_id());
    }
    fn unlock(&self, more_lock_config: MoreLockConfig, blocker: &Blocker) {
        self.is_locked.store(false, Ordering::Release);

        more_lock_config.disable(blocker);

        println!("Unlocked Group (Id: {})", self.as_id());
    }

    fn lock_when_blocked(
        self: &Arc<Self>,
        lock_config: LockConfig,
        blocker: &Blocker
    ) -> LockWhenBlocked 
    {
        match lock_config {
            LockConfig::NoLock => None.into(),
            LockConfig::LockWhenBlocked(more_lock_config) => Some(more_lock_config).into(),
            LockConfig::Timer(timer, more_lock_config) => {
                let self_cloned = self.clone();
                let blocker = blocker.clone();

                task::spawn(async move {
                    self_cloned.lock_with_timer(timer, more_lock_config, &blocker).await;
                });

                Some(more_lock_config).into()
            }
        }
    }

    async fn block_task<F, Fut>(self: Arc<Self>, apps: Vec<&App>, task: F) 
    where
        F: FnOnce(Arc<Self>, Vec<&'static App>) -> Fut,
        Fut: Future + Send + 'static,
        <Fut as Future>::Output: Send
    {
        // REASON FOR UNSAFE: `task::spawn` requires captured values to be `'static`, therefore `&'static` is required
        // SAFETY: `apps` holds references into the `Arc<[App]>` owned by `self`.
        // `self` is an `Arc`, so moving it does not move the underlying data.
        // Both `apps` and `self` are moved into the spawned task, sharing the same scope, 
        // so `self` is guaranteed to outlive `apps`. `App` is `'static`, so no internal
        // references exist that could be invalidated. Transmute is sound.
        let apps: Vec<&App> = unsafe { std::mem::transmute(apps) };

        task::spawn(
            task(self, apps)
        );
    }
}

impl AsId for Group {
    fn as_id(&self) -> Id {
        self.id
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
pub struct SavedGroup {
    id: usize,
    app_names: Vec<String>,
    block_rule_opt: Option<SavedBlockConfig>,
}

impl SavedGroup {
    pub fn new(group: &Group, block_rule_opt: Option<BlockConfig>) -> Self {
        let app_names = group.app_names_owned();

        let saved_block_rule_opt = block_rule_opt
            .as_ref()
            .map(|b_r| SavedBlockConfig::from_block_rule(b_r));

        SavedGroup {
            id: group.id.as_usize(),
            app_names,
            block_rule_opt: saved_block_rule_opt,
        }
    }
}


