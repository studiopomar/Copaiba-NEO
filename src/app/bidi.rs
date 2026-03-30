/// Arabic text shaping and visual reordering for egui.
///
/// egui renders text left-to-right with no Unicode Bidi support and no
/// OpenType shaping. We pre-process Arabic strings so they display correctly:
///
///   1. Collect each "Arabic run": contiguous Arabic letters + spaces between
///      them (spaces are part of RTL text and their order must be reversed too).
///   2. Pass the whole run to `ar_reshaper` so contextual forms and ligatures
///      are computed with full cross-word context.
///   3. Reverse *all* characters of the shaped run.
///      This simultaneously reverses word order AND character order within
///      each word, which is exactly what an LTR renderer needs to display
///      RTL text correctly.
///
/// Non-Arabic segments (ASCII, numbers, emoji…) are left untouched.

use ar_reshaper::ArabicReshaper;

thread_local! {
    static RESHAPER: ArabicReshaper = ArabicReshaper::default();
}

/// Reshape an Arabic string for display in egui's LTR text renderer.
pub fn reshape(text: &str) -> String {
    if !needs_reshape(text) {
        return text.to_string();
    }

    RESHAPER.with(|reshaper| {
        let mut result = String::with_capacity(text.len() + 16);
        // We accumulate chars that belong to an "Arabic run".
        // A run includes Arabic letters AND spaces/punctuation between them,
        // so that word order is reversed together with the characters.
        let mut run = String::new();
        // Track whether the run contains at least one Arabic letter
        // (so lone spaces at the start are not pulled into a run).
        let mut run_has_arabic = false;

        let flush = |run: &mut String, has_arabic: &mut bool, out: &mut String| {
            if run.is_empty() { return; }
            if *has_arabic {
                // Reshape the whole run (cross-word context) then reverse chars.
                let shaped = reshaper.reshape(run.trim_end());
                let trailing_spaces = run.len() - run.trim_end().len();
                for ch in shaped.chars().rev() {
                    out.push(ch);
                }
                // preserve trailing spaces (now they become leading after reversal,
                // but for display purposes we push them after the reversed Arabic)
                for _ in 0..trailing_spaces {
                    out.push(' ');
                }
            } else {
                // run has no Arabic letters (e.g. leading/trailing spaces)
                out.push_str(run);
            }
            run.clear();
            *has_arabic = false;
        };

        for ch in text.chars() {
            if is_arabic(ch) {
                run.push(ch);
                run_has_arabic = true;
            } else if ch == ' ' && run_has_arabic {
                // Space between Arabic words — keep it in the run so word
                // order is reversed together with the letters.
                run.push(ch);
            } else {
                // Non-Arabic, non-inter-word space: flush current run.
                flush(&mut run, &mut run_has_arabic, &mut result);
                result.push(ch);
            }
        }
        flush(&mut run, &mut run_has_arabic, &mut result);
        result
    })
}

/// True if the string contains at least one Arabic letter.
fn needs_reshape(text: &str) -> bool {
    text.chars().any(is_arabic)
}

/// Arabic script Unicode blocks (basic + supplement + presentation forms).
fn is_arabic(ch: char) -> bool {
    matches!(ch,
        '\u{0600}'..='\u{06FF}' |
        '\u{0750}'..='\u{077F}' |
        '\u{08A0}'..='\u{08FF}' |
        '\u{FB50}'..='\u{FDFF}' |
        '\u{FE70}'..='\u{FEFF}'
    )
}
