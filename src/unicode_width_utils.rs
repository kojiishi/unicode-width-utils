use std::sync::LazyLock;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

static IS_CJK: LazyLock<bool> = LazyLock::new(|| match std::env::var("UNICODE_WIDTH") {
    Ok(value) => value.eq_ignore_ascii_case("cjk"),
    _ => false,
});

#[derive(Clone, Copy, Debug)]
pub struct UnicodeWidth {
    is_cjk: bool,
}

impl Default for UnicodeWidth {
    fn default() -> Self {
        Self { is_cjk: *IS_CJK }
    }
}

impl UnicodeWidth {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cjk(is_cjk: bool) -> Self {
        Self { is_cjk }
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
}
