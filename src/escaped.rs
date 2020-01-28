// This file implements helpers for tag escaping and unescaping. Each function
// lists all the possible escape characters, even if the fallback would be
// valid.
//
// See https://ircv3.net/specs/extensions/message-tags.html for more
// information.

// Takes a single character and returns a str representing what to replace it
// with or None if it can be used verbatim.
#[inline(always)]
pub(crate) fn escape_char(c: char) -> Option<&'static str> {
    match c {
        ';' => Some(r"\:"),
        ' ' => Some(r"\s"),
        '\\' => Some(r"\\"),
        '\r' => Some(r"\r"),
        '\n' => Some(r"\n"),
        _ => None,
    }
}

// Takes the char after an escape character and returns what it should be
// decoded as.
#[inline(always)]
pub(crate) fn unescape_char(c: char) -> char {
    match c {
        ':' => ';',
        's' => ' ',
        '\\' => '\\',
        'r' => '\r',
        'n' => '\n',

        // Fallback should just drop the escaping.
        _ => c,
    }
}
