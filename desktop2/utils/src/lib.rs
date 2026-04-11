
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
