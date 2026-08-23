use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub enum Error {
    AlreadyInit,
    UninitID,
}

pub trait AsId {
    fn as_id(&self) -> Id;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Id(usize);

impl Id {
    pub fn init() -> Result<(), Error> {
        if NEXT_ID.load(Ordering::Relaxed) != 0 {
            return Err(Error::AlreadyInit);
        }
        NEXT_ID.store(1, Ordering::Relaxed);

        Ok(())
    }

    pub fn init_with(all: &[impl AsId]) -> Result<(), Error> {
        if NEXT_ID.load(Ordering::Relaxed) != 0 {
            return Err(Error::AlreadyInit);
        }

        let mut highest_id = 1;

        for a in all {
            let id = a.as_id().as_usize();

            if id >= highest_id {
                highest_id = id + 1;
            }
        }

        NEXT_ID.store(highest_id, Ordering::Relaxed);

        Ok(())
    }

    pub fn new_unique_id() -> Result<Id, Error> {
        if NEXT_ID.load(Ordering::Relaxed) == 0 {
            return Err(Error::UninitID);
        }

        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        Ok(Id(id))
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for Id {
    fn from(from: usize) -> Self {
        Self(from)
    }
}

impl AsId for Id {
    fn as_id(&self) -> Id {
        *self
    }
}
