#[macro_export]
macro_rules! vm_runtime_error {
    ($($t: tt)+) => {
        {
            println!("\x1b[31m\x1b[1mVM ERROR:\x1b[0m {}", format!($($t)+));
            std::process::exit(0);
        }
    };
}
