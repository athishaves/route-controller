//! Compile-time logging for macro expansion

/// Logs messages only when ROUTE_CONTROLLER_VERBOSE environment variable is set during compilation
macro_rules! log_verbose {
	($($arg:tt)*) => {
		if option_env!("ROUTE_CONTROLLER_VERBOSE").is_some() {
			println!($($arg)*);
		}
	};
}

/// Always logs messages during compilation
macro_rules! log_info {
	($($arg:tt)*) => {
		println!($($arg)*);
	};
}

pub(crate) use log_verbose;
