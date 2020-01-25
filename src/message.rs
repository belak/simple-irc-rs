use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::option::Option;

use super::error::ParseError;

#[derive(Debug, PartialEq, Default)]
pub struct Message<'a> {
    pub tags: BTreeMap<&'a str, String>,
    pub prefix: Option<&'a str>,
    pub command: &'a str,
    pub params: Vec<&'a str>,
}

fn parse_tags<'a>(input: &'a str) -> Result<BTreeMap<&'a str, String>, ParseError> {
    let mut ret = BTreeMap::new();

    for tag_data in input.split(";") {
        let (tag_name, raw_tag_value) = if let Some(loc) = tag_data.find("=") {
            (&tag_data[..loc], tag_data.get(loc + 1..).unwrap_or(""))
        } else {
            // If there's no equals sign, we need to default to the empty
            // string/
            (tag_data, "")
        };

        let mut tag_value = String::new();
        let mut tag_value_chars = raw_tag_value.chars();
        while let Some(c) = tag_value_chars.next() {
            if c == '\\' {
                match tag_value_chars.next() {
                    Some(escaped_char) => tag_value.push(match escaped_char {
                        ':' => ';',
                        's' => ' ',
                        '\\' => '\\',
                        'r' => '\r',
                        'n' => '\n',

                        // Fallback should just drop the escaping.
                        _ => escaped_char,
                    }),

                    // None at this point means we're at the end of the value,
                    // so we can drop it.
                    None => {}
                }
            } else {
                tag_value.push(c);
            }
        }

        ret.insert(tag_name, tag_value);
    }

    Ok(ret)
}

impl<'a> TryFrom<&'a str> for Message<'a> {
    type Error = ParseError;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        // We want a mutable input so we can jump through it as we parse the
        // message.
        let mut input = input;

        if input.ends_with("\n") {
            input = &input[..input.len() - 1];
        }
        if input.ends_with("\r") {
            input = &input[..input.len() - 1];
        }

        let mut tags: Option<BTreeMap<&'a str, String>> = None;
        let mut prefix: Option<&'a str> = None;

        if input.get(..1) == Some("@") {
            // Find the first space so we can split on it.
            if let Some(loc) = input.find(" ") {
                let tag_data = &input[1..loc];
                tags = Some(parse_tags(tag_data)?);

                // Update input to point to everything after the space
                input = &input[loc..];
            } else {
                return Err(ParseError::TagError("failed to parse tag data".to_string()));
            }
        }

        input = input.trim_start_matches(" ");

        if input.get(..1) == Some(":") {
            // Find the first space so we can split on it.
            if let Some(loc) = input.find(" ") {
                prefix = Some(&input[1..loc]);

                // Update input to point to everything after the space
                input = &input[loc..];
            } else {
                return Err(ParseError::PrefixError(
                    "failed to parse prefix data".to_string(),
                ));
            }
        }

        // Parse out the params
        let mut params = Vec::new();
        loop {
            // Drop any leading spaces
            input = input.trim_start_matches(" ");

            match input.get(..1) {
                // If a param started with a :, that means the rest of the input
                // is a single trailing param.
                Some(":") => {
                    params.push(&input[1..]);
                    break;
                }

                // Anything else is a normal param.
                Some(_) => {
                    match input.find(" ") {
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

        if params.len() == 0 {
            return Err(ParseError::CommandError("command not found".to_string()));
        }

        // Take the first param as the command. Note that we've already checked
        // if params is empty, so we can unwrap this safely.
        let (command, params) = params.split_first().unwrap();

        Ok(Message {
            tags: tags.unwrap_or(BTreeMap::new()),
            prefix: prefix,
            command: *command,
            params: params.to_vec(),
        })
    }
}
