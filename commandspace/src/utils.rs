/// dbg but only in debug builds
#[macro_export]
macro_rules! _dbg {
    ($($val:expr),+ $(,)?) => {{
        #[cfg(debug_assertions)]
        {
            $(dbg!(&$val);)+
        }
    }};
    ($($args:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            dbg!($($args)*)
        }
    }};
}
