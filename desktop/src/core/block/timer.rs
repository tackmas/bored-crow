use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::mem;

use chrono::{DateTime, FixedOffset, Local};

use serde::{Deserialize, Serialize};

use tokio::task;
use tokio::time::{self, Duration, Interval};

use crate::platform::Blocker;

use super::{App, CommonBlockInfo, Group, MoreLockConfig};
use super::block_rule::LockWhenBlocked;

#[derive(Clone, Serialize, Deserialize)]
pub struct Timer {
    date_time: DateTime<Local>,
    duration: Duration,
}

impl Timer {
    pub fn zero() -> Self {
        Self {
            date_time: Local::now(),
            duration: Duration::ZERO,
        }
    }
    pub fn new(duration: Duration) -> Self {
        Self {
            date_time: Local::now(),
            duration,
        }
    }
    pub fn when_done(&self) -> DateTime<Local> {
        self.date_time + self.duration
    }
}

impl Group {
    pub(super) async fn block_with_timer(
        self: Arc<Self>,
        timer: Timer,
        lock_when_blocked: LockWhenBlocked,
        cbi: CommonBlockInfo,
    ) {
        let now = Local::now();

        let diff = timer.when_done() - now;

        if diff.num_seconds() > 0 {
            if let Some(more_lock_config) = *lock_when_blocked {
                self.lock(more_lock_config, &cbi.blocker);
            }

            self.block_apps(&cbi.blocker).await;

            let diff_std = diff
                .to_std()
                .expect("if statement gurantees positive duration");

            println!("{diff_std:?}");

            task::spawn(async move {
                let cbi = cbi;

                tokio::select! {
                    res = cbi.unblock_rx => {
                        res.unwrap();
                    },
                    _ = time::sleep(diff_std) => {
                        println!("Block Timer completed")
                    }
                }
                if let Some(more_lock_config) = *lock_when_blocked {
                    self.unlock(more_lock_config, &cbi.blocker);
                } 

                self.unblock_apps(&cbi.blocker).await;

                self.clear_unblocker();
            });

        } else {
            self.clear_unblocker();
        }
    }

    pub(super) async fn lock_with_timer(
        &self, 
        lock: Timer, 
        more_lock_config: MoreLockConfig,
        blocker: &Blocker
    ) {
        let now = Local::now();

        let diff = lock.when_done() - now;

        if diff.num_seconds() > 0 {
            self.lock(more_lock_config, blocker);

            println!("Locking");

            let diff_std = diff
                .to_std()
                .expect("if statement gurantees positive duration");

            time::sleep(diff_std).await;

            println!("Lock Timer completed");

            self.unlock(more_lock_config, blocker);
        }
    }
}
