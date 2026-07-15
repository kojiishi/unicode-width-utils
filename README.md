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

- **Configuration Object**: Provides the configuration object for
  various different needs.
  - The tab size and whether to expand them to spaces or not.
  - The size of control characters.
  - Whether to use alternate width calculation
    more suited for CJK contexts or not.
    It controls East Asian Ambiguous characters
    (such as Greek, Cyrillic, and some symbol characters)
    to be 1 or 2 columns wide.
- **Environment Variable Support**: Initialize the default CJK mode
  using the `UNICODE_WIDTH=cjk` environment variable,
  so that end users can change it to match their environment
  for when applications don't explicitly specify it.
- **Safe Truncation**: Truncate strings to a specific column width
  without breaking UTF-8 characters,
  including optional tab support.

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
    // Treat ambiguous characters as 1 column wide
    let non_cjk = UnicodeWidth::with_cjk(false);
    assert_eq!(non_cjk.char('█'), 1);

    // Treat ambiguous characters as 2 columns wide (CJK mode)
    let cjk = UnicodeWidth::with_cjk(true);
    assert_eq!(cjk.char('█'), 2);
}
```

### Global Default Configuration

You can change the default configuration
for future instances created via `UnicodeWidth::new()`:
```rust
use unicode_width_utils::UnicodeWidth;

fn main() {
    // Globally set the default mode to CJK
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
    let uw = UnicodeWidth::with_cjk(false);
    assert_eq!(uw.truncate("hello", 3), "hel");

    let cjk = UnicodeWidth::with_cjk(true);
    // 'あ' is 2 columns wide
    assert_eq!(cjk.truncate("あああ", 3), "あ");
    assert_eq!(cjk.truncate("A█B", 2), "A");
}
```

Please see the [documentation][docs] for more details.

## License

Licensed under the Apache License, Version 2.0.

[`unicode-width` crate]: https://crates.io/crates/unicode-width
