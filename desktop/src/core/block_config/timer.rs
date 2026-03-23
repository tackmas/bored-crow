use tokio::{
    time::{
        self,
        Duration,
    },
};

use crate::platform::Blocker;

use super::{
    Group,
};

impl Group {
    pub async fn block_with_timer(&self, until_unblock: Duration, blocker: Blocker) {
        
        let apps = &self.apps;

        blocker.block_vec(apps).unwrap();
        self.lock();

        time::sleep(until_unblock).await;

        blocker.unblock_vec(apps).unwrap();
        self.unlock();
    }
}

