pub mod core;
pub mod platform;
pub mod ui;



use std::ops::{
    Deref,
    DerefMut,
};

use tokio::sync::oneshot;

use ui::{gui, gui2};

#[macro_export]
macro_rules! unwrap_variant {
    ($self:expr, $variant:path) => {
        match $self {
            $variant(inner) => inner,
            _ => panic!("wrong enum variant"), 
        }
    };
}

#[macro_export]
macro_rules! match_into_closure {
    ($(self:expr),+, $($variant:path),+) => {
        
    };
}


pub struct MyOption<T>(Option<T>);

