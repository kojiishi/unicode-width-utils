[![CI-badge]][CI]
[![crate-badge]][crate]
[![docs-badge]][docs]

[CI-badge]: https://github.com/kojiishi/unicode-width-utils/actions/workflows/rust-ci.yml/badge.svg
[CI]: https://github.com/kojiishi/unicode-width-utils/actions/workflows/rust-ci.yml
[crate-badge]: https://img.shields.io/crates/v/unicode-width-utils.svg
[crate]: https://crates.io/crates/unicode-width-utils
[docs-badge]: https://docs.rs/unicode-width-utils/badge.svg
[docs]: https://docs.rs/unicode-width-utils/

# unicode-width-utils

A thin-wrapper for the [`unicode-width` crate] with additional functionalities,
such as dynamic configuration for CJK (East Asian Ambiguous) widths and
safe string truncation.

## Features

- **Configuration Object**: Provides a configuration object
  that is easy to pass around
  for various different needs.
  - The tab size and whether to expand them to spaces or not.
  - The size of control characters.
  - Whether to use alternate width calculation
    more suited for CJK contexts or not.
    It controls East Asian Ambiguous characters
    (such as Greek, Cyrillic, and some symbol characters)
    to be 1 or 2 columns wide.
  - Also support the `UNICODE_WIDTH=cjk` environment variable
    to initialize the default CJK mode.
    This enables end users in CJK contexts
    to change the default mode to match their environments.
- **Safe Truncation**: Truncate strings to a specific column width
  without breaking UTF-8 characters,
  including optional tab support.
- **Line Wrapping**: Wrap strings to multiple lines at a specific column.

## Installation
```bash
cargo add unicode-width-utils
```

## Usage

### Basic Example

```rust
use unicode_width_utils::UnicodeWidth;

fn main() {
    let uw = UnicodeWidth::new();
    assert_eq!(uw.char('A'), 1);
    assert_eq!(uw.str("Hello"), 5);
}
```

### CJK Ambiguous Widths

You can explicitly configure whether East Asian Ambiguous characters are treated
as 1 or 2 columns wide:
```rust
use unicode_width_utils::UnicodeWidth;

fn main() {
    // Treat ambiguous characters as 1 column wide.
    let mut uw = UnicodeWidth::with_cjk(false);
    assert_eq!(uw.char('█'), 1);

    // Treat ambiguous characters as 2 columns wide (CJK mode).
    uw.set_cjk(true);
    assert_eq!(cjk.char('█'), 2);
}
```

### Global Default Configuration

You can change the default configuration
for future instances created via `UnicodeWidth::new()`:
```rust
use unicode_width_utils::UnicodeWidth;

fn main() {
    // Globally set the default mode to CJK.
    UnicodeWidth::set_default_cjk(true);
    
    let uw = UnicodeWidth::new();
    assert_eq!(uw.char('█'), 2);
}
```

Alternatively, you can initialize the default CJK mode
via the environment variable:
```bash
UNICODE_WIDTH=cjk
```

### String Truncation

Truncate a string slice so that its total display width does not exceed a maximum limit:

```rust
use unicode_width_utils::UnicodeWidth;

fn main() {
    let mut uw = UnicodeWidth::new();
    assert_eq!(uw.truncate("hello", 3), "hel");
    // 'あ' is 2 columns wide.
    assert_eq!(uw.truncate("あああ", 3), "あ");

    uw.set_tab_size(4);
    assert_eq!(uw.truncate("A\tB", 4), Cow::Owned::<str>("A\t".into()));
    uw.set_expand_tab(true);
    assert_eq!(uw.truncate("A\tB", 4), Cow::Owned::<str>("A   ".into()));
}
```

### Line Wrapping

```rust
use unicode_width_utils::UnicodeWidth;

fn main() {
    let uw = UnicodeWidth::new();
    let lines: Vec<_> = uw.lines("12345678", 3).collect();
    assert_eq!(lines, vec!["123", "456", "78"]);
}
```

Please see the [documentation][docs] for more details,
and [releases] for the change history.

## License

Licensed under the Apache License, Version 2.0.

[releases]: https://github.com/kojiishi/unicode-width-utils/releases
[`unicode-width` crate]: https://crates.io/crates/unicode-width
