use std::num::NonZeroU8;

use chrono::Weekday;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WeekdaySet(NonZeroU8);

impl WeekdaySet {
    fn is_empty(self) -> bool {
        let as_u8 = u8::from(self.0); 

        as_u8 & 0b1111_1110u8 == 0
    }
}

impl Iterator for WeekdaySet {
    type Item = Weekday;

    fn next(&mut self) -> Option<Self::Item> {
        
    }
}