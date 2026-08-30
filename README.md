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
such as line wrapping and truncation.

## Features

- **Configurations**: Provides various configurations,
  stored in a lightweight configuration object that is easy to pass around.
  - The **tab size** and whether to expand them to spaces or not.
  - The **size of control characters**.
  - Treat **ANSI escape sequences** zero-width
    (requires the optional `ansi` feature).
  - Use alternate width calculation more **suited for CJK contexts**.
- **Safe Truncation**: Truncate strings to a specific column width
  without breaking UTF-8 characters,
  including optional tab support.
- **Line Wrapping**: Wrap strings to multiple lines at a specific column.
- **Unicode Segmentation**: Truncate and wrap strings
  only at Unicode grapheme cluster boundaries
  (requires the optional `segment` feature).

## Installation
```bash
cargo add unicode-width-utils
```

### Cargo Features

- `ansi`: Enables support for making ANSI escape sequences zero-width.
  This feature is optional and disabled by default.
  To enable it, add the feature:
  ```shell
  cargo add unicode-width-utils --features ansi
  ```
- `segment`: Enables support for truncating and wrapping lines
  only at Unicode grapheme boundaries.
  This feature is optional and disabled by default.
  To enable it, add the feature:
  ```shell
  cargo add unicode-width-utils --features segment
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

### Tab Characters

Tab characters can be 1 column wide or a jump to the next tab stop.
You can configure the tab size,
along with whether to convert them to spaces or not.

```rust
use unicode_width_utils::UnicodeWidth;

fn main() {
    let mut uw = UnicodeWidth::new();
    assert_eq!(uw.str("A\tBC"), 4);
    uw.set_tab_size(4);
    assert_eq!(uw.str("A\tBC"), 6);
    assert_eq!(uw.truncate("A\tBC", 5), "A\tB");
    uw.set_expand_tab(true);
    assert_eq!(uw.truncate("A\tBC", 5), "A   B");
}
```

### ANSI Escape Sequences

You can configure whether to make ANSI escape sequences zero-width or not.

> [!NOTE]
> This requires the `ansi` feature to be enabled.

```rust
use unicode_width_utils::UnicodeWidth;

fn main() {
    let mut uw = UnicodeWidth::new();
    let input = "A\x1B[31mZZ";
    assert_eq!(uw.str(input), 8);
    uw.set_ansi(true);
    assert_eq!(uw.str(input), 3);
    assert_eq!(uw.truncate(input, 2), Cow::Borrowed("A\x1B[31mZ"));
}
```

### Unicode Grapheme Segmentation

When the `segment` feature is enabled,
line truncation and wrapping will always occur
only at Unicode grapheme boundaries.

> [!NOTE]
> This requires the `segment` feature to be enabled.

```rust
use unicode_width_utils::UnicodeWidth;

fn main() {
    let uw = UnicodeWidth::new();
    // "a\u{301}" is a single grapheme cluster (a with combining acute accent) of width 1.
    // It cannot be truncated to width 0 without breaking the grapheme cluster.
    assert_eq!(uw.truncate("a\u{301}b", 1), "a\u{301}");
    assert_eq!(uw.truncate("a\u{301}b", 0), "");
}
```

### CJK Ambiguous Widths

You can explicitly configure whether East Asian Ambiguous characters are treated
as 1 or 2 columns wide.
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

> [!NOTE]
> The default setting is `false` for `new()` and `default()`,
> but setting the environment variable `UNICODE_WIDTH=cjk` turns it to `true`.

### String Truncation

Truncate a string slice so that its total display width does not exceed a
maximum limit.

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

[`lines()`] can create an iterator of multiple lines by wrapping a string.

```rust
use unicode_width_utils::UnicodeWidth;

fn main() {
    let uw = UnicodeWidth::new();
    assert_eq!(
        uw.lines("12345678", 3).collect::<Vec<_>>(),
        vec!["123", "456", "78"]
    );
}
```

Please see the [documentation][docs] for more details,
[releases] for the change history,
or [file issues][issues] if any.

## License

Licensed under the Apache License, Version 2.0.

[issues]: https://github.com/kojiishi/unicode-width-utils/issues
[`lines()`]: https://docs.rs/unicode-width-utils/latest/unicode_width_utils/struct.UnicodeWidth.html#method.lines
[releases]: https://github.com/kojiishi/unicode-width-utils/releases
[`unicode-width` crate]: https://crates.io/crates/unicode-width
