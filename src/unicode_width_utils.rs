use std::borrow::Cow;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::WidthIterator;

static IS_CJK: LazyLock<AtomicBool> = LazyLock::new(|| {
    let is_cjk = match std::env::var("UNICODE_WIDTH") {
        Ok(value) => value.eq_ignore_ascii_case("cjk"),
        _ => false,
    };
    AtomicBool::new(is_cjk)
});

/// A configuration helper to measure character and string widths.
///
/// It determines the width of Unicode characters and strings,
/// optionally treating East Asian Ambiguous characters
/// (such as certain Greek, Cyrillic, and CJK characters)
/// as having a width of 2 (CJK mode).
///
/// The default CJK mode is initialized at startup
/// based on the `UNICODE_WIDTH` environment variable,
/// but can also be dynamically modified using [`set_default_cjk()`].
///
/// [`set_default_cjk()`]: UnicodeWidth::set_default_cjk
#[derive(Clone, Debug)]
pub struct UnicodeWidth {
    is_cjk: bool,
    pub(crate) should_expand_tab: bool,
    pub(crate) tab_size: u8,
    pub(crate) control_size: u8,
}

impl Default for UnicodeWidth {
    fn default() -> Self {
        Self {
            is_cjk: IS_CJK.load(Ordering::Relaxed),
            should_expand_tab: false,
            tab_size: 0,
            control_size: 1,
        }
    }
}

impl UnicodeWidth {
    /// Create a `UnicodeWidth` instance using the default CJK mode.
    ///
    /// The default CJK mode is determined by the global setting,
    /// which defaults to the value of the `UNICODE_WIDTH` environment variable
    /// (value `"cjk"` enabling CJK mode).
    ///
    /// See also [`set_default_cjk()`].
    ///
    /// [`set_default_cjk()`]: UnicodeWidth::set_default_cjk
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new `UnicodeWidth` instance with a specific CJK flag.
    ///
    /// If `is_cjk` is true,
    /// East Asian Ambiguous characters will be treated as 2 columns wide.
    /// If false, they will be treated as 1 column wide.
    ///
    /// # Examples
    /// ```
    /// use unicode_width_utils::UnicodeWidth;
    ///
    /// let non_cjk = UnicodeWidth::with_cjk(false);
    /// assert_eq!(non_cjk.char('█'), 1);
    ///
    /// let cjk = UnicodeWidth::with_cjk(true);
    /// assert_eq!(cjk.char('█'), 2);
    /// ```
    pub fn with_cjk(is_cjk: bool) -> Self {
        Self {
            is_cjk,
            ..Default::default()
        }
    }

    /// Set the default CJK configuration dynamically.
    ///
    /// Future instances
    /// created using [`UnicodeWidth::new()`] or [`UnicodeWidth::default()`]
    /// will inherit this default value
    /// unless explicitly overridden with [`with_cjk()`].
    ///
    /// [`with_cjk()`]: `UnicodeWidth::with_cjk`
    /// # Examples
    /// ```
    /// use unicode_width_utils::UnicodeWidth;
    ///
    /// // Set default CJK mode to true.
    /// UnicodeWidth::set_default_cjk(true);
    /// let uw = UnicodeWidth::new();
    /// assert_eq!(uw.char('█'), 2);
    ///
    /// // Set default CJK mode to false.
    /// UnicodeWidth::set_default_cjk(false);
    /// let uw2 = UnicodeWidth::new();
    /// assert_eq!(uw2.char('█'), 1);
    /// ```
    pub fn set_default_cjk(is_cjk: bool) {
        IS_CJK.store(is_cjk, Ordering::Relaxed);
    }

    /// Set the size of control characters.
    ///
    /// # Examples
    /// ```
    /// use unicode_width_utils::UnicodeWidth;
    ///
    /// let mut uw = UnicodeWidth::new();
    /// assert_eq!(uw.char('\t'), 1);
    /// assert_eq!(uw.str("A\tB"), 3);
    /// uw.set_control_size(0);
    /// assert_eq!(uw.char('\t'), 0);
    /// assert_eq!(uw.str("A\tB"), 2);
    /// ```
    pub fn set_control_size(&mut self, size: u8) {
        self.control_size = size;
    }

    /// Set the tab size.
    /// Initially `0`.
    ///
    /// This setting is used by [`truncate()`] and [`UnicodeWidth::str()`].
    ///
    /// See also [`set_expand_tab()`].
    ///
    /// [`set_expand_tab()`]: UnicodeWidth::set_expand_tab
    /// [`truncate()`]: UnicodeWidth::truncate
    ///
    /// # Examples
    /// ```
    /// # use std::borrow::Cow;
    /// use unicode_width_utils::UnicodeWidth;
    ///
    /// let mut uw = UnicodeWidth::new();
    /// uw.set_tab_size(4);
    /// assert_eq!(uw.truncate("A\tB", 3), Cow::Borrowed("A"));
    /// assert_eq!(uw.truncate("A\tB", 4), Cow::Borrowed("A\t"));
    /// assert_eq!(uw.truncate("A\tB", 5), Cow::Borrowed("A\tB"));
    /// ```
    pub fn set_tab_size(&mut self, tab_size: u8) {
        self.tab_size = tab_size;
    }

    /// Set whether tabs should be expanded to spaces.
    /// Initially `false`.
    ///
    /// This setting is used by [`truncate()`] and [`UnicodeWidth::str()`].
    ///
    /// See also [`set_tab_size()`].
    ///
    /// [`set_tab_size()`]: UnicodeWidth::set_tab_size
    /// [`truncate()`]: UnicodeWidth::truncate
    ///
    /// # Examples
    /// ```
    /// # use std::borrow::Cow;
    /// use unicode_width_utils::UnicodeWidth;
    ///
    /// let mut uw = UnicodeWidth::new();
    /// uw.set_tab_size(4);
    /// uw.set_expand_tab(true);
    /// assert_eq!(uw.truncate("A\tB", 3), Cow::Borrowed("A"));
    /// assert_eq!(uw.truncate("A\tB", 4), Cow::Owned::<str>("A   ".into()));
    /// assert_eq!(uw.truncate("A\tB", 5), Cow::Owned::<str>("A   B".into()));
    /// ```
    pub fn set_expand_tab(&mut self, should_expand_tab: bool) {
        self.should_expand_tab = should_expand_tab;
    }

    /// Return the column width of a character.
    ///
    /// This is a wrapper of [`UnicodeWidthChar`].
    /// It calls `width` or `width_cjk` depending on the configuration.
    ///
    /// Control characters return `1` by default
    /// to match [`UnicodeWidthStr`].
    /// See also [`char_opt()`] and [`set_control_size()`] for control characters.
    ///
    /// [`char_opt()`]: UnicodeWidth::char_opt
    /// [`set_control_size()`]: UnicodeWidth::set_control_size
    ///
    /// # Examples
    /// ```
    /// use unicode_width_utils::UnicodeWidth;
    ///
    /// let uw = UnicodeWidth::new();
    /// assert_eq!(uw.char('A'), 1);
    /// assert_eq!(uw.char('あ'), 2);
    /// ```
    pub fn char(&self, ch: char) -> usize {
        self.char_opt(ch).unwrap_or(self.control_size as usize)
    }

    /// Return the column width of a character.
    ///
    /// Return `None` for control characters
    /// or other characters without a defined width.
    ///
    /// This is a wrapper of [`UnicodeWidthChar`].
    /// It calls `width` or `width_cjk` depending on the configuration.
    ///
    /// [`UnicodeWidthChar`]: https://docs.rs/unicode-width/latest/unicode_width/trait.UnicodeWidthChar.html
    ///
    /// # Examples
    /// ```
    /// use unicode_width_utils::UnicodeWidth;
    ///
    /// let uw = UnicodeWidth::new();
    /// assert_eq!(uw.char_opt('A'), Some(1));
    /// assert_eq!(uw.char_opt('\n'), None);
    /// ```
    pub fn char_opt(&self, ch: char) -> Option<usize> {
        match self.is_cjk {
            false => UnicodeWidthChar::width(ch),
            true => UnicodeWidthChar::width_cjk(ch),
        }
    }

    /// Return the total column width of a string.
    ///
    /// This is a wrapper of [`UnicodeWidthStr`].
    /// It calls `width` or `width_cjk` depending on the configuration,
    /// unless the [tab size is set][`set_tab_size()`]
    /// or the [control character size is set][`set_control_size()`],
    /// in which case the internal logic computes the width
    /// by calling [`char_opt()`] or [`char()`] repeatedly.
    ///
    /// [`char()`]: UnicodeWidth::char
    /// [`char_opt()`]: UnicodeWidth::char_opt
    /// [`set_control_size()`]: UnicodeWidth::set_control_size
    /// [`set_tab_size()`]: UnicodeWidth::set_tab_size
    /// [`truncate()`]: UnicodeWidth::truncate
    /// [`UnicodeWidthStr`]: https://docs.rs/unicode-width/latest/unicode_width/trait.UnicodeWidthStr.html
    ///
    /// # Examples
    /// ```
    /// use unicode_width_utils::UnicodeWidth;
    ///
    /// let mut uw = UnicodeWidth::new();
    /// assert_eq!(uw.str("Hello"), 5);
    /// assert_eq!(uw.str("Hello\t"), 6);
    /// uw.set_tab_size(4);
    /// assert_eq!(uw.str("Hello\t"), 8);
    /// ```
    pub fn str(&self, str: &str) -> usize {
        if self.tab_size > 0 {
            let mut iter = WidthIterator::new(self, str);
            iter.consume_all();
            return iter.width();
        }
        if self.control_size != 1 {
            return str.chars().map(|ch| self.char(ch)).sum();
        }
        match self.is_cjk {
            false => UnicodeWidthStr::width(str),
            true => UnicodeWidthStr::width_cjk(str),
        }
    }

    /// Truncate a string slice to a maximum column width.
    ///
    /// The returned slice will be the longest prefix of `input`
    /// whose total column width does not exceed `max_width`.
    ///
    /// See also [`set_tab_size()`] and [`set_expand_tab()`].
    ///
    /// [`set_tab_size()`]: UnicodeWidth::set_tab_size
    /// [`set_expand_tab()`]: UnicodeWidth::set_expand_tab
    ///
    /// # Examples
    /// ```
    /// use unicode_width_utils::UnicodeWidth;
    ///
    /// let uw = UnicodeWidth::new();
    /// assert_eq!(uw.truncate("hello", 3), "hel");
    ///
    /// // Truncating CJK text (each 'あ' is 2 columns wide).
    /// assert_eq!(uw.truncate("あああ", 3), "あ");
    /// assert_eq!(uw.truncate("A█B", 2), "A█");
    ///
    /// let cjk = UnicodeWidth::with_cjk(true);
    /// assert_eq!(cjk.truncate("A█B", 2), "A");
    /// ```
    pub fn truncate<'a>(&self, input: &'a str, max_width: usize) -> Cow<'a, str> {
        let mut iter = WidthIterator::new(self, input);
        iter.set_max_width(max_width);
        iter.consume_all();
        iter.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char() {
        let uw = UnicodeWidth::with_cjk(false);
        let cjk = UnicodeWidth::with_cjk(true);
        assert_eq!(uw.char('A'), 1);
        assert_eq!(cjk.char('A'), 1);
        assert_eq!(uw.char('\u{2588}'), 1);
        assert_eq!(cjk.char('\u{2588}'), 2);
        assert_eq!(uw.char('\u{3042}'), 2);
        assert_eq!(cjk.char('\u{3042}'), 2);
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
    fn str_tab() {
        let mut uw = UnicodeWidth::with_cjk(false);
        uw.set_tab_size(4);
        assert_eq!(uw.str("A"), 1);
        assert_eq!(uw.str("A\t"), 4);
        assert_eq!(uw.str("A\tB"), 5);
        assert_eq!(uw.str("A\t\tB"), 9);
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

        // Control characters with 1 column wide.
        assert_eq!(uw.truncate("A\nB", 1), "A");
        assert_eq!(uw.truncate("A\nB", 2), "A\n");
        assert_eq!(uw.truncate("\nA", 0), "");
        assert_eq!(uw.truncate("\nA", 1), "\n");
    }

    #[test]
    fn default_cjk() {
        let original = IS_CJK.load(Ordering::Relaxed);

        UnicodeWidth::set_default_cjk(false);
        assert_eq!(UnicodeWidth::default().char('\u{2588}'), 1);
        assert_eq!(UnicodeWidth::new().char('\u{2588}'), 1);

        UnicodeWidth::set_default_cjk(true);
        assert_eq!(UnicodeWidth::default().char('\u{2588}'), 2);
        assert_eq!(UnicodeWidth::new().char('\u{2588}'), 2);

        UnicodeWidth::set_default_cjk(original);
    }

    // If `tab_size` = 0, tab behaves as 1 column wide control character.
    #[test]
    fn truncate_tabs_no_tab_size() {
        let mut uw = UnicodeWidth::new();
        for _ in 0..2 {
            assert_eq!(uw.truncate("A\tB", 1), Cow::Borrowed("A"));
            assert_eq!(uw.truncate("A\tB", 2), Cow::Borrowed("A\t"));
            assert_eq!(uw.truncate("A\tB", 3), Cow::Borrowed("A\tB"));
            uw.set_expand_tab(true);
        }
    }

    #[test]
    fn truncate_tabs_expand_multi() {
        let mut uw = UnicodeWidth::new();
        uw.set_tab_size(4);
        uw.set_expand_tab(true);
        assert_eq!(uw.truncate("\t\t", 7), Cow::Owned::<str>("    ".into()));
        assert_eq!(uw.truncate("\t\t", 8), Cow::Owned::<str>("        ".into()));
    }
}
