use std::fmt::Display;

auto trait NotEqUnit {}
impl !NotEqUnit for () {
}

pub trait MaybeDisplay {
    fn into_solution(self) -> Option<String>;
}
impl<T: Display + NotEqUnit> MaybeDisplay for T {
    fn into_solution(self) -> Option<String> {
        Some(self.to_string())
    }
}
impl MaybeDisplay for () {
    fn into_solution(self) -> Option<String> {
        None
    }
}
