pub mod group;
pub mod ipc;
pub mod platform;
pub mod saved;
pub mod macros;

use std::cmp::{Ordering as CmpOrdering};
use std::sync::atomic::{AtomicUsize, Ordering};

pub const USERNAME: &str = "Tackmas";
pub const APP_NAME: &str = "Bored Crow";

static PROHIBIT_UNINSTALL_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn can_uninstall() -> bool {
    let flag = PROHIBIT_UNINSTALL_COUNT.load(Ordering::SeqCst) == 0;

    println!("Can uninstall is {flag}");

    flag
}

pub fn increase_prohibit_uninstall_count() {
    let old = PROHIBIT_UNINSTALL_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("Increased prohibit uninstall count. New value: {}", old + 1)
}

pub fn decrease_prohibit_uninstall_count() {
    let old = PROHIBIT_UNINSTALL_COUNT.fetch_sub(1, Ordering::SeqCst);
    println!("Decreased prohibit uninstall count. New value: {}", old - 1)
}


// Items from here forth is just miscellanous, convenience stuff that does not
// fit in specific modules
pub fn ordering_by_alphabetical(a: &str, b: &str) -> CmpOrdering {
    let (mut a_bytes, mut b_bytes) = (a.bytes(), b.bytes());
    loop {
        match (a_bytes.next(), b_bytes.next()) {
            (Some(a_byte), Some(b_byte)) => {
                let case_insensitive_ordering = a_byte.to_ascii_lowercase()
                    .cmp(&b_byte.to_ascii_lowercase());

                if case_insensitive_ordering != CmpOrdering::Equal {
                    return case_insensitive_ordering;
                } 

                let case_sensitive_ordering = b_byte.cmp(&a_byte);

                if case_sensitive_ordering != CmpOrdering::Equal {
                    return case_sensitive_ordering;
                }
            },
            (Some(_), None) => return CmpOrdering::Greater,
            (None, Some(_)) => return CmpOrdering::Less,
            (None, None) => return CmpOrdering::Equal
        } 
    }
}