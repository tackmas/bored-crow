pub mod core;
pub mod gui;
pub mod ipc;
pub mod platform;
pub mod utils;

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

fn lol(lol: &mut String) {
    aa(lol);

    lol.push_str("aa");
} 



fn aa(lol: &mut String) {
    lol.push_str("a")
}