pub mod block_config;
pub mod id;

use std::borrow::Borrow;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::oneshot;

use crate::platform::{Blocker, ProcessName};
use crate::saved::{Group as SavedGroup};

pub use self::id::{AsId, Id};
pub use self::block_config::{
    BlockConfig, BlockRuleKind, CustomWeek, LockConfig, LockWhenBlocked, 
    MoreLockConfig, Timer, TimeRange, TimeRangesOnWeek, UniformWeekdays, 
    WeekSchedule, WeekScheduleT
};

pub struct Group {
    pub id: Id,
    pub process_names: Vec<ProcessName>,
    unblocker_tx: Mutex<Option<Unblocker>>,
    is_locked: AtomicBool,
}

impl Group {
    pub fn new_with_process_names(process_names: Vec<ProcessName>) -> Self {
        let id = Id::new_unique_id().unwrap();
        let unblocker_tx = Mutex::new(None);
        let is_locked = AtomicBool::new(false);

        Group {
            id,
            process_names,
            unblocker_tx,
            is_locked,
        }
    }

    pub async fn from_saved(saved_group: impl Borrow<SavedGroup>, blocker: &Blocker) -> Arc<Self> {
        let saved_group = saved_group.borrow();
        let id = saved_group.id;

        let process_names = saved_group.apps
            .iter()
            .map(|app| ProcessName::from(app.clone()))
            .collect();

        let block_rule_opt = saved_group.block_config
            .clone()
            .map(|block_rule_kind| BlockConfig::from_saved(block_rule_kind, id));

        let group = Arc::new(Self {
            id,
            process_names,
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

    pub fn is_blocked(&self) -> bool {
        let is_blocked = self.unblocker_tx.lock().unwrap().is_some();

        println!("Is blocked: '{is_blocked}'");

        is_blocked
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked.load(Ordering::Acquire)
    }

    pub async fn block(self: Arc<Self>, block_config: BlockConfig, blocker: Blocker) {
        let (unblocker_tx, unblocker_rx) = oneshot::channel::<()>();
        
        let mut guard = self.unblocker_tx.lock().unwrap();

        if guard.is_some() {
            eprintln!("Already blocked");

            return;
        }

        *guard = Some(Unblocker::new(unblocker_tx));

        println!("Set 'Unblocker' to 'Some'");

        drop(guard);

        println!("Blocking Group (Id: {})", self.as_id());

        block_config.kind.block_group(self, blocker, unblocker_rx);
    }
    pub fn unblock(&self) -> Result<(), &'static str> {
        let is_locked = self.is_locked();

        if is_locked {
            eprintln!("Group is locked");
            return Err("group is locked");
        }

        let Some(unblocker_tx) = self.take_unblocker() else {
            println!("Already is not blocked (Group::unblock)");

            return Ok(())
        };
        unblocker_tx.unblock();

        println!("Unblocked Group (Id: {})", self.as_id());

        Ok(())
    }

    // Forcibly unblocks the group, even if it is locked
    pub fn force_unblock(&self) {
        let Some(unblocker_tx) = self.take_unblocker() else {
            println!("Already is not blocked (Group::force_unblock)");
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


}

impl AsId for Group {
    fn as_id(&self) -> Id {
        self.id
    }
}

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
