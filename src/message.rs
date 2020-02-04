use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write;
use std::option::Option;
use std::str::FromStr;

use super::error::Error;

use crate::escaped::{escape_char, unescape_char};

#[derive(Debug, PartialEq, Default)]
pub struct Prefix {
    pub nick: String,
    pub user: Option<String>,
    pub host: Option<String>,
}

impl Prefix {
    pub fn new(nick: &str) -> Self {
        Self::new_with_all(nick, None, None)
    }

    pub fn new_with_all(nick: &str, user: Option<&str>, host: Option<&str>) -> Self {
        Prefix {
            nick: nick.to_string(),
            user: user.map(|s| s.to_string()),
            host: host.map(|s| s.to_string()),
        }
    }
}

impl FromStr for Prefix {
    type Err = Error;

    // nickname [ [ "!" user ] "@" host ]
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts = input.splitn(2, '@');

        // Split on host first
        let rest = parts.next().unwrap_or("");
        let host = parts.next();

        let mut parts = rest.splitn(2, '!');
        let nick = parts.next().unwrap_or("").to_string();
        let user = parts.next();

        Ok(Prefix {
            nick,
            user: user.and_then(|s| {
                if s == "" {
                    None
                } else {
                    Some(s.to_string())
                }
            }),
            host: host.and_then(|s| {
                if s == "" {
                    None
                } else {
                    Some(s.to_string())
                }
            }),
        })
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.nick)?;

        if let Some(user) = self.user.as_ref() {
            f.write_char('!')?;
            f.write_str(&user[..])?;
        }

        if let Some(host) = self.host.as_ref() {
            f.write_char('@')?;
            f.write_str(&host[..])?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Message {
    pub tags: BTreeMap<String, String>,
    pub prefix: Option<Prefix>,
    pub command: String,
    pub params: Vec<String>,
}

impl Message {
    pub fn new(command: String, params: Vec<String>) -> Self {
        Message {
            command,
            params,
            ..Default::default()
        }
    }

    pub fn new_with_all(
        tags: BTreeMap<String, String>,
        prefix: Option<Prefix>,
        command: String,
        params: Vec<String>,
    ) -> Self {
        Message {
            tags,
            prefix,
            command,
            params,
        }
    }

    pub fn new_with_prefix(command: String, params: Vec<String>, prefix: Prefix) -> Self {
        Message {
            prefix: Some(prefix),
            command,
            params,
            ..Default::default()
        }
    }
}

fn parse_tags(input: &str) -> Result<BTreeMap<String, String>, Error> {
    let mut tags = BTreeMap::new();

    for tag_data in input.split(';') {
        let mut pieces = tag_data.splitn(2, '=');
        let tag_name = pieces
            .next()
            .ok_or_else(|| Error::TagError("missing tag name".to_string()))?;
        let raw_tag_value = pieces.next().unwrap_or("");

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

        tags.insert(tag_name.to_string(), tag_value);
    }

    Ok(tags)
}

impl FromStr for Message {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

        if input.starts_with('@') {
            let mut parts = (&input[1..]).splitn(2, ' ');
            let tag_data = parts
                .next()
                .ok_or_else(|| Error::TagError("missing tag data".to_string()))?;

            tags = parse_tags(tag_data)?;

            // Either advance to the next token, or return an empty string.
            input = parts.next().unwrap_or("").trim_start_matches(' ');
        }

        if input.starts_with(':') {
            let mut parts = (&input[1..]).splitn(2, ' ');
            prefix = Some(
                parts
                    .next()
                    .ok_or_else(|| Error::TagError("missing prefix data".to_string()))?
                    .parse()
                    .or_else(|_| Err(Error::TagError("failed to parse prefix data".to_string())))?,
            );

            // Either advance to the next token, or return an empty string.
            input = parts.next().unwrap_or("").trim_start_matches(' ');
        }

        let mut parts = input.splitn(2, ' ');
        let command = parts
            .next()
            .ok_or_else(|| Error::CommandError("missing command".to_string()))?
            .to_string();

        // Either advance to the next token, or return an empty string.
        input = parts.next().unwrap_or("").trim_start_matches(' ');

        // Parse out the params
        let mut params = Vec::new();
        while !input.is_empty() {
            // Special case - if the param starts with a :, it's a trailing
            // param, so we need to include the rest of the input as the param.
            if input.starts_with(':') {
                params.push(input[1..].to_string());
                break;
            }

            let mut parts = input.splitn(2, ' ');
            if let Some(param) = parts.next() {
                params.push(param.to_string());
            }

            // Either advance to the next token, or return an empty string.
            input = parts.next().unwrap_or("").trim_start_matches(' ');
        }

        Ok(Message {
            tags,
            prefix,
            command,
            params,
        })
    }
}

impl fmt::Display for Message {
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
            prefix.fmt(f)?;
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
