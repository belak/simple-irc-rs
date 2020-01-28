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
