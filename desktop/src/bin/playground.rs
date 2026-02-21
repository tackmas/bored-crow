trait A<T> {
    fn a(self) -> Self;
}

impl<T> A<T> for T {
    fn a(self) -> Self {
        self
    }
}


fn main() {
    println!("Hello world")
}