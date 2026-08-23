macro_rules! inner {
    ($($inner:expr),*) => {
        inner
    };
}

#[macro_export]
macro_rules! unwrap_variant {
    ($self:expr, $variant:path => $($variable:ident),+) => {
        match $self {
            $variant($($variable),+) => ($($variable),+),
            _ => panic!("Wrong enum variant"),
        }
    };
}

#[macro_export]
macro_rules! is_variant {
    ($self:expr, $variant:pat) => {
        match $self {
            $variant => true,
            _ => false
        }
    };
}

#[macro_export]
macro_rules! match_into_closure {
    ($(self:expr),+, $($variant:path),+) => {};
}
