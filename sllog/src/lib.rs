#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        {
            use std::env;
            if let Some(value) = env::var_os("LOG") {
                let val = value
                    .to_string_lossy()
                    .parse::<i32>()
                    .expect("The value should be an integer (1..4)");

                if val >= 1 {
                    println!("[TRACE]: {}", format_args!($($arg)*));
                }
            }
        }
    };
}
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        {
            use std::env;
            if let Some(value) = env::var_os("LOG") {
                let val = value
                    .to_string_lossy()
                    .parse::<i32>()
                    .expect("The value should be an integer (1..4)");

                if val >= 2 {
                    println!("\x1B[32m[INFO]: {}\x1B[0m", format_args!($($arg)*));
                }
            }
        }
    };
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        {
            use std::env;
            if let Some(value) = env::var_os("LOG") {
                let val = value
                    .to_string_lossy()
                    .parse::<i32>()
                    .expect("The value should be an integer (1..4)");

                if val >= 3 {
                    println!("\x1B[33m[WARN]: {}\x1B[0m", format_args!($($arg)*));
                }
            }
        }
    };
}
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        {
            use std::env;
            if let Some(value) = env::var_os("LOG") {
                let val = value
                    .to_string_lossy()
                    .parse::<i32>()
                    .expect("The value should be an integer (1..4)");

                if val >= 4 {
                    println!("\x1B[31m[ERROR]: {}\x1B[0m", format_args!($($arg)*));
                }
            }
        }
    };
}