#[macro_export]
macro_rules! wrapper_with_deref {
    ($wrapper_type_name:ident, $wrapped_type:ty) => {
        pub struct $wrapper_type_name($wrapped_type);

        impl std::ops::Deref for $wrapper_type_name {
            type Target = $wrapped_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        
        impl std::ops::DerefMut for $wrapper_type_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

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


