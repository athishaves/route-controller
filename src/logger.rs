//! Compile-time logging for macro expansion

/// Logs messages only when verbose-logging feature is enabled
macro_rules! log_verbose {
	($($arg:tt)*) => {
		#[cfg(feature = "verbose-logging")]
		{
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
