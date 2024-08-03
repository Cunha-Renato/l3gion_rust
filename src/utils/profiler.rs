#[macro_export]
macro_rules! profiler_begin {
    () => {
        // #[cfg(debug_assertions)]
        optick::start_capture();
    }    
}

#[macro_export]
macro_rules! profiler_end {
    ($_path:tt) => {
        // #[cfg(debug_assertions)]
        optick::stop_capture($_path);
    };
}

#[macro_export]
// #[cfg(debug_assertions)]
macro_rules! profile_function {
    () => {
        let _func_name = optick::function!();
        optick::event!(_func_name);
    };
}
// #[macro_export]
// #[cfg(not(debug_assertions))]
// macro_rules! profile_function {() => {};}

#[macro_export]
// #[cfg(debug_assertions)]
macro_rules! profile_scope {
    ($_name:tt) => {
        optick::event!($_name);
    };
}
// #[macro_export]
// #[cfg(not(debug_assertions))]
// macro_rules! profile_scope {($_name:tt) => {};}