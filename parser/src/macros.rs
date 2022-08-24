#[macro_export]
macro_rules! or {
    ($first:expr, $($typ:tt)+) => {
        or($first, or!($($typ)+))
    };

    ($last:expr) => {
        $last
    }
}
