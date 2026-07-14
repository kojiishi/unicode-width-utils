use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

static IS_CJK: LazyLock<AtomicBool> = LazyLock::new(|| {
    let is_cjk = match std::env::var("UNICODE_WIDTH") {
        Ok(value) => value.eq_ignore_ascii_case("cjk"),
        _ => false,
    };
    AtomicBool::new(is_cjk)
});

#[derive(Clone, Copy, Debug)]
pub struct UnicodeWidth {
    is_cjk: bool,
}

impl Default for UnicodeWidth {
    fn default() -> Self {
        Self {
            is_cjk: IS_CJK.load(Ordering::Relaxed),
        }
    }
}

impl UnicodeWidth {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cjk(is_cjk: bool) -> Self {
        Self { is_cjk }
    }

    pub fn set_default_cjk(is_cjk: bool) {
        IS_CJK.store(is_cjk, Ordering::Relaxed);
    }

    pub fn char(&self, ch: char) -> Option<usize> {
        match self.is_cjk {
            false => UnicodeWidthChar::width(ch),
            true => UnicodeWidthChar::width_cjk(ch),
        }
    }

    pub fn str(&self, str: &str) -> usize {
        match self.is_cjk {
            false => UnicodeWidthStr::width(str),
            true => UnicodeWidthStr::width_cjk(str),
        }
    }

    pub fn truncate<'a>(&self, str: &'a str, max_width: usize) -> &'a str {
        let mut width = 0;
        for (i, ch) in str.char_indices() {
            let ch_width = self.char(ch).unwrap_or(0);
            width += ch_width;
            if width > max_width {
                return &str[..i];
            }
        }
        str
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char() {
        let uw = UnicodeWidth::with_cjk(false);
        let cjk = UnicodeWidth::with_cjk(true);
        assert_eq!(uw.char('A'), Some(1));
        assert_eq!(cjk.char('A'), Some(1));
        assert_eq!(uw.char('\u{2588}'), Some(1));
        assert_eq!(cjk.char('\u{2588}'), Some(2));
        assert_eq!(uw.char('\u{3042}'), Some(2));
        assert_eq!(cjk.char('\u{3042}'), Some(2));
    }

    #[test]
    fn str() {
        let uw = UnicodeWidth::with_cjk(false);
        let cjk = UnicodeWidth::with_cjk(true);
        assert_eq!(uw.str("A"), 1);
        assert_eq!(cjk.str("A"), 1);
        assert_eq!(uw.str("\u{2588}"), 1);
        assert_eq!(cjk.str("\u{2588}"), 2);
        assert_eq!(uw.str("\u{3042}"), 2);
        assert_eq!(cjk.str("\u{3042}"), 2);
    }

    #[test]
    fn truncate() {
        let uw = UnicodeWidth::with_cjk(false);
        let cjk = UnicodeWidth::with_cjk(true);

        assert_eq!(uw.truncate("hello", 0), "");
        assert_eq!(uw.truncate("hello", 4), "hell");
        assert_eq!(uw.truncate("hello", 5), "hello");
        assert_eq!(uw.truncate("hello", 6), "hello");
        assert_eq!(uw.truncate("hello", 10), "hello");

        // \u{2588} is 1 column wide for `uw`, and 2 columns wide for `cjk`.
        assert_eq!(uw.truncate("A\u{2588}B", 2), "A\u{2588}");
        assert_eq!(cjk.truncate("A\u{2588}B", 2), "A");

        // \u{3042} ('あ') is 2 columns wide.
        assert_eq!(uw.truncate("\u{3042}", 1), "");
        assert_eq!(uw.truncate("\u{3042}", 2), "\u{3042}");
        assert_eq!(uw.truncate("\u{3042}", 3), "\u{3042}");

        // Control characters with 0 width.
        assert_eq!(uw.truncate("A\nB", 1), "A\n");
        assert_eq!(uw.truncate("A\nB", 2), "A\nB");
        assert_eq!(uw.truncate("\nA", 0), "\n");
    }

    #[test]
    fn default_cjk() {
        let original = IS_CJK.load(Ordering::Relaxed);

        UnicodeWidth::set_default_cjk(false);
        assert_eq!(UnicodeWidth::default().char('\u{2588}'), Some(1));
        assert_eq!(UnicodeWidth::new().char('\u{2588}'), Some(1));

        UnicodeWidth::set_default_cjk(true);
        assert_eq!(UnicodeWidth::default().char('\u{2588}'), Some(2));
        assert_eq!(UnicodeWidth::new().char('\u{2588}'), Some(2));

        UnicodeWidth::set_default_cjk(original);
    }
}
