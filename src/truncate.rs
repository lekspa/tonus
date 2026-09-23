use unicode_width::UnicodeWidthChar;

// NOTE:
// Strings may contain emoji and other symbols that occcupy multiple bytes.
// unicode_width crate is required to get correct string index to slice at

/// Truncate text to fit `max` terminal cells, appending custom ellipsis if cut.
pub fn truncate_with_ellipsis(text: &str, max_width: usize, ellipsis: &str) -> String {
    if max_width == 0 || text.is_empty() {
        return String::new();
    }

    let elipsis_width = ellipsis
        .chars()
        .map(|c| c.width().unwrap_or(0))
        .sum::<usize>();

    if elipsis_width > max_width {
        return String::new();
    }

    let mut width = 0;
    let mut end = 0;

    for (index, character) in text.char_indices() {
        let character_width = character.width().unwrap_or(0);

        if width + character_width + elipsis_width > max_width {
            break;
        }

        width += character_width;
        // Find where the character ends to slice at correct position and avoid panic.
        end = index + character.len_utf8();
    }

    if end == text.len() {
        text.to_owned()
    } else {
        format!("{}{}", &text[..end], ellipsis)
    }
}

pub fn truncate(text: &str, max_width: usize) -> String {
    truncate_with_ellipsis(text, max_width, "…")
}
