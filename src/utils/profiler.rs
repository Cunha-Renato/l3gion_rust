#[macro_export]
macro_rules! profiler_begin {
    () => {
        #[cfg(debug_assertions)]
        optick::start_capture();
    }    
}

#[macro_export]
macro_rules! profiler_end {
    ($path:tt) => {
        #[cfg(debug_assertions)]
        optick::stop_capture($path);
    };
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! profile_function {
    () => {
        let func_name = optick::function!();
        optick::event!(func_name);
    };
}
#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! profile_function {() => {};}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! profile_scope {
    ($name:tt) => {
        optick::event!($name);
    };
}
#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! profile_scope {($name:tt) => {};}