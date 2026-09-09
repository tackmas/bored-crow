use std::sync::Arc;

use chrono::{DateTime, Local};

use serde::{Deserialize, Serialize};

use tokio::sync::oneshot;
use tokio::task;
use tokio::time::{self, Duration};

use crate::platform::Blocker;

use super::{Group, LockWhenBlocked, MoreLockConfig};

#[derive(Clone, Debug, Deserialize, Serialize)]
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

    pub(super) async fn block_group(
        self,
        group: Arc<Group>,
        lock_when_blocked: LockWhenBlocked,
        blocker: Blocker,
        unblocker_rx: oneshot::Receiver<()>
    ) {
        let now = Local::now();

        let diff = self.when_done() - now;

        if diff.num_seconds() > 0 {
            if let Some(more_lock_config) = lock_when_blocked.flag {
                group.lock(more_lock_config, &blocker);
            }

            let process_names = group.process_names.iter().cloned();
            blocker.block_processes(process_names).await;

            let diff_std = diff
                .to_std()
                .expect("if statement gurantees positive duration");

            println!("{diff_std:?}");

            task::spawn(async move {
                tokio::select! {
                    res = unblocker_rx => {
                        res.unwrap();
                    },
                    _ = time::sleep(diff_std) => {
                        println!("Block Timer completed")
                    }
                }
                if let Some(more_lock_config) = lock_when_blocked.flag {
                    group.unlock(more_lock_config, &blocker);
                } 

                let process_names = group.process_names.iter().cloned();
                blocker.unblock_processes(process_names).await;

                group.clear_unblocker();
            });

        } else {
            group.clear_unblocker();
        }
    }

    pub(super) async fn lock_timer(
        self, 
        group: &Group, 
        more_lock_config: MoreLockConfig,
        blocker: &Blocker
    ) {
        let now = Local::now();

        let diff = self.when_done() - now;

        if diff.num_seconds() > 0 {
            group.lock(more_lock_config, blocker);

            println!("Locking");

            let diff_std = diff
                .to_std()
                .expect("if statement gurantees positive duration");

            time::sleep(diff_std).await;

            println!("Lock Timer completed");

            group.unlock(more_lock_config, blocker);
        }
    }    
}


