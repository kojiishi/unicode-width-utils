//! A thin-wrapper for the [`unicode-width` crate]
//! with additional functionalities.
//!
//! This crate provides [`UnicodeWidth`]
//! which can dynamically measure character and string widths
//! based on whether they should be treated as CJK (East Asian Ambiguous) widths.
//!
//! It also provides a helper method to truncate strings to a maximum display width.
//!
//! # Examples
//! ```
//! use unicode_width_utils::UnicodeWidth;
//!
//! // Create an instance with the default CJK setting.
//! let uw = UnicodeWidth::new();
//! assert_eq!(uw.char('A'), Some(1));
//!
//! // Explicitly specify CJK behavior.
//! let non_cjk = UnicodeWidth::with_cjk(false);
//! let cjk = UnicodeWidth::with_cjk(true);
//!
//! // Ambiguous CJK characters (like '█') have width 1 or 2.
//! assert_eq!(non_cjk.char('█'), Some(1));
//! assert_eq!(cjk.char('█'), Some(2));
//!
//! // Truncate a string to a maximum width of columns.
//! let truncated = cjk.truncate("A█B", 2);
//! assert_eq!(truncated, "A");
//! ```
//!
//! [`unicode-width` crate]: https://crates.io/crates/unicode-width

mod unicode_width_utils;

pub use unicode_width_utils::UnicodeWidth;
