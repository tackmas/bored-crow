use std::{
    sync::{
        atomic::{
            AtomicBool
        }
    }
};

use serde::{
    Serialize,
    Deserialize,
};

use tokio::{
    time::{
        self,
        Duration,
        Interval,
    },
};

use platform::Blocker;

use super::{
    App,
    Group,
};



impl Group {
    pub async fn block_with_timer(
        &self, 
        until_unblock: Duration, 
        apps: Vec<&App>,
        blocker: Blocker,
        lock_when_blocked: bool
    ) {
        self.set_is_locked(lock_when_blocked);
        blocker.block_vec(&apps).await.unwrap();

        time::sleep(until_unblock).await;

        self.unlock();
        blocker.unblock_vec(&apps).await.unwrap();;
    }
}

