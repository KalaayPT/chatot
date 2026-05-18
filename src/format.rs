use std::sync::OnceLock;

use serde_derive::Deserialize;

use crate::charmap::Charmap;
use crate::error::FormatError;

pub const DIALOG_LINE_MAX_PX: u32 = 200;

#[derive(Deserialize)]
struct GlyphWidthData {
    #[serde(rename = "glyphWidths")]
    glyph_widths: Vec<u8>,
}

static GLYPH_WIDTHS: OnceLock<Vec<u8>> = OnceLock::new();

pub fn default_glyph_widths() -> &'static [u8] {
    GLYPH_WIDTHS.get_or_init(|| {
        let data: GlyphWidthData = serde_json::from_str(include_str!("../glyph_widths.json"))
            .expect("Failed to parse embedded glyph widths");
        data.glyph_widths
    })
}

/// Width of the glyph for `code`, falling back to `fallback` when the code has
/// no entry in the width table.
///
/// pokeplatinum indexes the width table at `charcode - 1` (see
/// `FontManager_CalcStringWidth` in `src/font_manager.c`): code 0 is
/// `CHAR_NONE`, which is not a real glyph, so the first glyph is code 1.
fn lookup_width(code: u16, widths: &[u8], fallback: u32) -> u32 {
    (code as usize)
        .checked_sub(1)
        .and_then(|i| widths.get(i))
        .map(|&w| w as u32)
        .unwrap_or(fallback)
}

/// A hard line break present in the source text.
enum BreakKind {
    /// `\n` — line feed. Parity-sensitive: it moves to the next line, which
    /// from the bottom line of the box can only be done by clearing (`\r`).
    Line,
    /// `\r` — clear the text box (and show the "press to continue" prompt).
    Clear,
    /// `\f` — scroll the text box one line.
    Scroll,
}

enum Item {
    /// A run of glyphs with no spaces or breaks, and its pixel width.
    Word(String, u32),
    /// A run of one or more spaces, and its total pixel width.
    Space(String, u32),
    /// A hard line break present in the source.
    HardBreak(BreakKind),
}

fn flush_word(items: &mut Vec<Item>, word: &mut String, word_px: &mut u32) {
    if !word.is_empty() {
        items.push(Item::Word(std::mem::take(word), *word_px));
        *word_px = 0;
    }
}

/// Split dialogue text into words, space runs and hard breaks, measuring the
/// pixel width of each glyph against `widths`.
///
/// Escape sequences are parsed the same way the encoder parses them
/// (`encode_string_to_message` in `src/encode.rs`): `\xXXXX` is a raw code,
/// `\n`/`\r`/`\f` are hard breaks, `[alias]` is a multi-character glyph and
/// `{...}` is a runtime command. Runtime commands have zero width since their
/// expansion is unknown until runtime. A raw `\x` escape that falls outside
/// the glyph width table is treated as a zero-width control code.
fn tokenize(text: &str, charmap: &Charmap, widths: &[u8]) -> Result<Vec<Item>, FormatError> {
    // pokeplatinum renders unknown characters as `?`, so its width is the
    // fallback for any character outside the glyph width table.
    let fallback = charmap
        .encode_map
        .get("?")
        .map(|&code| lookup_width(code, widths, 0))
        .unwrap_or(0);
    let space_px = charmap
        .encode_map
        .get(" ")
        .map(|&code| lookup_width(code, widths, fallback))
        .unwrap_or(fallback);

    let mut items: Vec<Item> = Vec::new();
    let mut chars = text.chars().peekable();
    let mut word = String::new();
    let mut word_px = 0u32;

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => match chars.next() {
                Some(brk @ ('n' | 'r' | 'f')) => {
                    flush_word(&mut items, &mut word, &mut word_px);
                    let kind = match brk {
                        'n' => BreakKind::Line,
                        'r' => BreakKind::Clear,
                        _ => BreakKind::Scroll,
                    };
                    items.push(Item::HardBreak(kind));
                }
                Some('x') => {
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        match chars.next() {
                            Some(h) => hex.push(h),
                            None => break,
                        }
                    }
                    if hex.len() != 4 {
                        return Err(FormatError::IncompleteHexEscape);
                    }
                    let code = u16::from_str_radix(&hex, 16)
                        .map_err(|_| FormatError::InvalidHexEscape(hex.clone()))?;
                    word.push_str("\\x");
                    word.push_str(&hex);
                    // A code outside the glyph table is a control code: zero width.
                    word_px += lookup_width(code, widths, 0);
                }
                Some(next) => {
                    let seq = format!("\\{next}");
                    let px = charmap
                        .encode_map
                        .get(&seq)
                        .map(|&code| lookup_width(code, widths, fallback))
                        .unwrap_or(fallback);
                    word.push_str(&seq);
                    word_px += px;
                }
                None => word.push('\\'),
            },
            ' ' => {
                flush_word(&mut items, &mut word, &mut word_px);
                let mut run = String::from(" ");
                while chars.peek() == Some(&' ') {
                    run.push(' ');
                    chars.next();
                }
                let px = space_px * run.len() as u32;
                items.push(Item::Space(run, px));
            }
            '[' => {
                let mut alias = String::from("[");
                let mut closed = false;
                while let Some(&next) = chars.peek() {
                    alias.push(next);
                    chars.next();
                    if next == ']' {
                        closed = true;
                        break;
                    }
                }
                let px = if closed {
                    charmap
                        .encode_map
                        .get(&alias)
                        .map(|&code| lookup_width(code, widths, fallback))
                        .unwrap_or(fallback)
                } else {
                    fallback
                };
                word.push_str(&alias);
                word_px += px;
            }
            '{' => {
                let mut cmd = String::from("{");
                while let Some(&next) = chars.peek() {
                    cmd.push(next);
                    chars.next();
                    if next == '}' {
                        break;
                    }
                }
                // Runtime command: zero width, value unknown until runtime.
                word.push_str(&cmd);
            }
            _ => {
                let px = charmap
                    .encode_map
                    .get(&ch.to_string())
                    .map(|&code| lookup_width(code, widths, fallback))
                    .unwrap_or(fallback);
                word.push(ch);
                word_px += px;
            }
        }
    }

    flush_word(&mut items, &mut word, &mut word_px);
    Ok(items)
}

/// Measure the pixel width of a line of dialogue text.
///
/// Hard breaks (`\n`, `\r`, `\f`) split the input into lines. Returns the
/// zero-based index of the widest line along with its pixel width, so callers
/// can report *which* line overflows. On a tie the earliest line wins. Callers
/// typically pass a single line at a time — that is the intended use — but a
/// multi-line string is handled gracefully.
///
/// Runtime commands (`{...}`) contribute zero width because their expanded
/// value is unknown until runtime, so a line containing one may still overflow
/// in game.
pub fn measure_line_width(
    text: &str,
    charmap: &Charmap,
    widths: &[u8],
) -> Result<(usize, u32), FormatError> {
    let items = tokenize(text, charmap, widths)?;

    let widest = items
        .split(|item| matches!(item, Item::HardBreak(_)))
        .map(|line| {
            line.iter()
                .map(|item| match item {
                    Item::Word(_, px) | Item::Space(_, px) => *px,
                    Item::HardBreak(_) => 0,
                })
                .sum::<u32>()
        })
        .enumerate()
        .fold(
            (0usize, 0u32),
            |best, (index, px)| {
                if px > best.1 { (index, px) } else { best }
            },
        );

    Ok(widest)
}

pub fn line_is_too_long(
    text: &str,
    charmap: &Charmap,
    widths: &[u8],
    max_line_px: u32,
) -> Result<bool, FormatError> {
    Ok(measure_line_width(text, charmap, widths)?.1 > max_line_px)
}

/// Wrap dialogue text so no line exceeds `max_line_px`, inserting soft line
/// breaks (`\n`) and box clears (`\r`) as needed.
///
/// The view alternates between the two lines of the text box: a break on the
/// top line emits `\n` (move to the bottom line), and a break on the bottom
/// line emits `\r` (clear the box). This applies to soft breaks *and* to hard
/// `\n` breaks already in the input — a hard `\n` reached while on the bottom
/// line is realised as `\r`, because a literal line feed there would overflow
/// the two-line box. Hard `\r` and `\f` breaks are emitted unchanged (they
/// clear/scroll regardless of view position). Runs of spaces are preserved
/// verbatim.
///
/// A trailing `\r` is appended (unless the text already ends in a hard break)
/// so the message ends with a "press to continue" prompt.
///
/// Returns [`FormatError::WordTooLong`] if a single word is wider than
/// `max_line_px`: such a word cannot fit on any line and cannot be split.
pub fn word_wrap(
    text: &str,
    charmap: &Charmap,
    widths: &[u8],
    max_line_px: u32,
) -> Result<String, FormatError> {
    let items = tokenize(text, charmap, widths)?;

    let mut result = String::new();
    let mut view_slot: u8 = 0;
    let mut line_px = 0u32;
    let mut at_line_start = true;
    let mut pending_space: Option<(String, u32)> = None;

    for item in items {
        match item {
            Item::HardBreak(kind) => {
                pending_space = None;
                match kind {
                    // A line feed from the bottom line of the box would
                    // overflow it, so realise it as a clear instead. This
                    // keeps the \n/\r alternation valid wherever soft-wrapping
                    // left the view.
                    BreakKind::Line if view_slot == 0 => {
                        result.push_str("\\n");
                        view_slot = 1;
                    }
                    BreakKind::Line | BreakKind::Clear => {
                        result.push_str("\\r");
                        view_slot = 0;
                    }
                    BreakKind::Scroll => {
                        result.push_str("\\f");
                        view_slot = 0;
                    }
                }
                line_px = 0;
                at_line_start = true;
            }
            Item::Space(text, px) => {
                pending_space = Some((text, px));
            }
            Item::Word(text, word_px) => {
                if word_px > max_line_px {
                    return Err(FormatError::WordTooLong {
                        word: text,
                        width: word_px,
                        max: max_line_px,
                    });
                }

                let (space_str, space_px) = if at_line_start {
                    pending_space = None;
                    (String::new(), 0)
                } else {
                    pending_space.take().unwrap_or((String::new(), 0))
                };

                if at_line_start || line_px + space_px + word_px <= max_line_px {
                    result.push_str(&space_str);
                    result.push_str(&text);
                    line_px += space_px + word_px;
                } else {
                    let soft = if view_slot == 0 { "\\n" } else { "\\r" };
                    result.push_str(soft);
                    view_slot = 1 - view_slot;
                    result.push_str(&text);
                    line_px = word_px;
                }
                at_line_start = false;
            }
        }
    }

    if !result.ends_with("\\r") && !result.ends_with("\\f") && !result.ends_with("\\n") {
        result.push_str("\\r");
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charmap::get_default_charmap;

    #[test]
    fn measure_line_width_empty() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        assert_eq!(measure_line_width("", cm, w).unwrap(), (0, 0));
    }

    #[test]
    fn measure_line_width_applies_glyph_index_offset() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        // 'i' and 'l' are narrow glyphs (3px and 4px) whose widths live at
        // code-1. Indexing without the offset would read different values.
        assert_eq!(measure_line_width("il", cm, w).unwrap().1, 7);
    }

    #[test]
    fn measure_line_width_hard_breaks_split_lines() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        assert_eq!(measure_line_width("\\n\\r\\f", cm, w).unwrap().1, 0);
    }

    #[test]
    fn measure_line_width_returns_widest_line() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        let short = measure_line_width("il", cm, w).unwrap().1;
        let long = measure_line_width("hello", cm, w).unwrap().1;
        assert!(long > short);
        // The widest line and its index are reported, wherever it sits.
        assert_eq!(measure_line_width("il\\nhello", cm, w).unwrap(), (1, long));
        assert_eq!(measure_line_width("hello\\nil", cm, w).unwrap(), (0, long));
    }

    #[test]
    fn measure_line_width_commands_are_zero() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        assert_eq!(
            measure_line_width("{STRVAR_1, 3, 0, 0}", cm, w).unwrap().1,
            0
        );
    }

    #[test]
    fn measure_line_width_control_codes_are_zero() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        // 0xFFFF is a control code with no glyph; it must not get a fallback width.
        assert_eq!(measure_line_width("\\xFFFF", cm, w).unwrap().1, 0);
    }

    #[test]
    fn measure_line_width_rejects_short_hex_escape() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        assert_eq!(
            measure_line_width("\\xAB", cm, w),
            Err(FormatError::IncompleteHexEscape)
        );
    }

    #[test]
    fn word_wrap_appends_trailing_r() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        assert_eq!(
            word_wrap("hello", cm, w, DIALOG_LINE_MAX_PX).unwrap(),
            "hello\\r"
        );
    }

    #[test]
    fn word_wrap_no_double_trailing_r() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        assert_eq!(
            word_wrap("hello\\r", cm, w, DIALOG_LINE_MAX_PX).unwrap(),
            "hello\\r"
        );
    }

    #[test]
    fn word_wrap_preserves_hard_break() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        assert_eq!(
            word_wrap("hi\\rbye", cm, w, DIALOG_LINE_MAX_PX).unwrap(),
            "hi\\rbye\\r"
        );
    }

    #[test]
    fn word_wrap_hard_break_keeps_alternation_after_soft_wrap() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        // "a b" soft-wraps "b" onto the bottom line, so the explicit \n that
        // follows arrives while on the bottom line. It must be realised as \r
        // (clear) — a second \n there would overflow the two-line box.
        assert_eq!(word_wrap("a b\\nc", cm, w, 10).unwrap(), "a\\nb\\rc\\r");
    }

    #[test]
    fn word_wrap_hard_break_on_top_line_stays_line_feed() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        // No soft-wrapping happens, so the hard \n is reached on the top line
        // and is emitted unchanged.
        assert_eq!(
            word_wrap("a\\nb", cm, w, DIALOG_LINE_MAX_PX).unwrap(),
            "a\\nb\\r"
        );
    }

    #[test]
    fn word_wrap_preserves_repeated_spaces() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        assert_eq!(
            word_wrap("a  b", cm, w, DIALOG_LINE_MAX_PX).unwrap(),
            "a  b\\r"
        );
    }

    #[test]
    fn word_wrap_splits_when_line_full() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        // "ab" and "cd" each fit, but together exceed the limit.
        assert_eq!(word_wrap("ab cd", cm, w, 20).unwrap(), "ab\\ncd\\r");
    }

    #[test]
    fn word_wrap_view_slot_cycles() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        // Each word fits alone but not paired, so the slot cycles n/r/n/r.
        assert_eq!(word_wrap("a b c d", cm, w, 10).unwrap(), "a\\nb\\rc\\nd\\r");
    }

    #[test]
    fn word_wrap_rejects_word_wider_than_line() {
        let cm = get_default_charmap();
        let w = default_glyph_widths();
        match word_wrap("ab", cm, w, 5) {
            Err(FormatError::WordTooLong { word, max, .. }) => {
                assert_eq!(word, "ab");
                assert_eq!(max, 5);
            }
            other => panic!("expected WordTooLong, got {other:?}"),
        }
    }
}
