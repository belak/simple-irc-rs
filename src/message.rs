use std::borrow::Cow;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Write;
use std::option::Option;

use super::error::ParseError;

use crate::escaped::{escape_char, unescape_char};

#[derive(Debug, PartialEq, Default)]
pub struct Message<'a> {
    pub tags: BTreeMap<&'a str, Cow<'a, str>>,
    pub prefix: Option<&'a str>,
    pub command: &'a str,
    pub params: Vec<&'a str>,
}

fn parse_tags<'a>(input: &'a str) -> Result<BTreeMap<&'a str, Cow<'a, str>>, ParseError> {
    let mut tags = BTreeMap::new();

    for tag_data in input.split(';') {
        let mut pieces = tag_data.splitn(2, '=');
        let tag_name = pieces
            .next()
            .ok_or_else(|| ParseError::TagError("missing tag name".to_string()))?;
        let raw_tag_value = pieces.next().unwrap_or("");

        // If the value doesn't contain any escaped characters, we can return
        // the string as-is.
        if !raw_tag_value.contains('\\') {
            tags.insert(tag_name, Cow::Borrowed(raw_tag_value));
            continue;
        }

        let mut tag_value = String::new();
        let mut tag_value_chars = raw_tag_value.chars();
        while let Some(c) = tag_value_chars.next() {
            if c == '\\' {
                if let Some(escaped_char) = tag_value_chars.next() {
                    tag_value.push(unescape_char(escaped_char));
                }
            } else {
                tag_value.push(c);
            }
        }

        tags.insert(tag_name, Cow::Owned(tag_value));
    }

    Ok(tags)
}

impl<'a> TryFrom<&'a str> for Message<'a> {
    type Error = ParseError;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        // We want a mutable input so we can jump through it as we parse the
        // message. Note that this shadows the input param on purpose so it
        // cannot accidentally be used later.
        let mut input = input;

        // Possibly chop off the ending \r\n where either of those characters is
        // optional.
        if input.ends_with('\n') {
            input = &input[..input.len() - 1];
        }
        if input.ends_with('\r') {
            input = &input[..input.len() - 1];
        }

        let mut tags = BTreeMap::new();
        let mut prefix = None;

        if input.get(..1) == Some("@") {
            // Find the first space so we can split on it.
            if let Some(loc) = input.find(' ') {
                let tag_data = &input[1..loc];
                tags = parse_tags(tag_data)?;

                // Update input to point to everything after the space
                input = &input[loc..];
            } else {
                return Err(ParseError::TagError("failed to parse tag data".to_string()));
            }

            // Trim up to the next valid character.
            input = input.trim_start_matches(' ');
        }

        if input.get(..1) == Some(":") {
            // Find the first space so we can split on it.
            if let Some(loc) = input.find(' ') {
                prefix = Some(&input[1..loc]);

                // Update input to point to everything after the space
                input = &input[loc..];
            } else {
                return Err(ParseError::PrefixError(
                    "failed to parse prefix data".to_string(),
                ));
            }

            // Note that we don't trim spaces here because that's handled by
            // param parsing.
        }

        // Parse out the params
        let mut params = Vec::new();
        loop {
            // Drop any leading spaces
            input = input.trim_start_matches(' ');

            match input.get(..1) {
                // If a param started with a :, that means the rest of the input
                // is a single trailing param.
                Some(":") => {
                    params.push(&input[1..]);
                    break;
                }

                // Anything else is a normal param.
                Some(_) => {
                    match input.find(' ') {
                        Some(loc) => {
                            params.push(&input[..loc]);
                            // Update input to point to everything after the space
                            input = &input[loc..];
                        }

                        // If we couldn't find a space, the rest of the string
                        // is the param.
                        None => {
                            params.push(input);
                            break;
                        }
                    }
                }

                // If we didn't get anything, this means we're at the end of the
                // string.
                None => {
                    break;
                }
            }
        }

        if params.is_empty() {
            return Err(ParseError::CommandError("command missing".to_string()));
        }

        // Take the first param as the command. Note that we've already checked
        // if params is empty, so we can unwrap this safely.
        let (command, params) = params.split_first().unwrap();

        Ok(Message {
            tags,
            prefix,
            command,
            params: params.to_vec(),
        })
    }
}

impl<'a> fmt::Display for Message<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.tags.is_empty() {
            f.write_char('@')?;

            for (i, (k, v)) in self.tags.iter().enumerate() {
                // We need to insert a separator for everything other than the
                // first value.
                if i != 0 {
                    f.write_char(';')?;
                }

                f.write_str(k)?;
                if v.is_empty() {
                    continue;
                }

                f.write_char('=')?;

                for c in v.chars() {
                    match escape_char(c) {
                        Some(escaped_str) => f.write_str(escaped_str)?,
                        None => f.write_char(c)?,
                    }
                }
            }

            f.write_char(' ')?;
        }

        if let Some(prefix) = &self.prefix {
            f.write_char(':')?;
            f.write_str(prefix)?;
            f.write_char(' ')?;
        }

        f.write_str(&self.command)?;

        if let Some((last, params)) = self.params.split_last() {
            for param in params {
                f.write_char(' ')?;
                f.write_str(param)?;
            }

            f.write_str(" :")?;
            f.write_str(last)?;
        }

        Ok(())
    }
}
