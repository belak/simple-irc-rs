use std::collections::BTreeMap;
use std::option::Option;

use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take},
    character::complete::line_ending,
    combinator::{all_consuming, opt, recognize},
    multi::{many0, separated_list},
    sequence::{delimited, pair, preceded},
    IResult, InputLength,
};

//use super::error::ParseError;

#[derive(Debug, PartialEq, Default)]
pub struct Message<'a> {
    pub tags: BTreeMap<&'a str, String>,
    pub prefix: Option<&'a str>,
    pub command: &'a str,
    pub params: Vec<&'a str>,
}

fn parse_nospcrlfcl(input: &str) -> IResult<&str, &str> {
    is_not("\0\r\n ")(input)
}

fn parse_tag_component(input: &str) -> IResult<&str, &str> {
    is_not("\0\r\n; ")(input)
}

fn parse_tag_name(input: &str) -> IResult<&str, &str> {
    is_not("\0\r\n= ")(input)
}

fn parse_escaped_char(input: &str) -> IResult<&str, &str> {
    let (input, escaped_char) = opt(take(1usize))(input)?;
    Ok((
        input,
        if let Some(escaped_char) = escaped_char {
            match escaped_char {
                ":" => ";",
                "s" => " ",
                "\\" => "\\",
                "r" => "\r",
                "n" => "\n",

                // Fallback should just drop the escaping.
                _ => escaped_char,
            }
        } else {
            // At the end of the string just drop the escape character.
            ""
        },
    ))
}

fn parse_empty_str(input: &str) -> IResult<&str, &str> {
    Ok((input, ""))
}

fn parse_tag_value(input: &str) -> IResult<&str, String> {
    let mut input = input;
    let mut ret = String::new();

    println!("VAL INPUT: {}", input);

    loop {
        // Try to consume non-escaped characters. If we find some, push them to
        // the ret string. Otherwise, move on.
        let (loop_input, consumed) = opt(is_not("\0\r\n\\ "))(input)?;
        if let Some(s) = consumed {
            ret.push_str(s);
        }

        // Once we get to the end of the input, we return.
        if loop_input.input_len() == 0 {
            return Ok((loop_input, ret));
        }

        // Try to parse an escape character
        let (loop_input, _) = tag("\\")(loop_input)?;

        // Parse the next single char
        let (loop_input, consumed) = parse_escaped_char(loop_input)?;
        ret.push_str(consumed);

        input = loop_input;
    }
}

fn parse_tags(input: &str) -> IResult<&str, BTreeMap<&str, String>> {
    // Split into a string with the tag list contents
    let (input, tags_input) = delimited(tag("@"), is_not("\0\r\n "), is_a(" "))(input)?;

    // Split into a list of name/value components
    let (_, tags_list) = all_consuming(separated_list(tag(";"), parse_tag_component))(tags_input)?;

    let mut ret = BTreeMap::new();

    // For each name/value component, split it and add it to the returned map.
    for tag_input in tags_list {
        let (tag_input, tag_name) = parse_tag_name(tag_input)?;

        let (tag_input, separator) = opt(tag("="))(tag_input)?;

        // If we found a separator, we need to parse a tag value.
        if let Some(_) = separator {
            let (_, tag_value) = all_consuming(parse_tag_value)(tag_input)?;
            ret.insert(tag_name, tag_value);
        } else {
            // Ensure we're at the end of the input.
            all_consuming(parse_empty_str)(tag_input)?;
            ret.insert(tag_name, "".to_string());
        }
    }

    Ok((input, ret))
}

fn parse_prefix(input: &str) -> IResult<&str, &str> {
    delimited(tag(":"), parse_nospcrlfcl, is_a(" "))(input)
}

fn parse_param(input: &str) -> IResult<&str, &str> {
    // NOTE: it's possible for the first is_not to consume all the input because
    // the latter parser is a subset of the former. Because of this, the second
    // matcher needs to be optional. recognize will take care of combining the
    // results anyway.
    recognize(pair(is_not("\0\r\n: "), opt(parse_nospcrlfcl)))(input)
}

fn parse_trailing(input: &str) -> IResult<&str, &str> {
    preceded(tag(" :"), alt((is_not("\r\n\0"), tag(""))))(input)
}

fn parse_params(input: &str) -> IResult<&str, Vec<&str>> {
    let (input, mut ret) = many0(preceded(is_a(" "), parse_param))(input)?;
    let (input, trailing) = opt(parse_trailing)(input)?;

    // If we had a trailing arg, append it to the params
    if let Some(s) = trailing {
        ret.push(s);
    }

    Ok((input, ret))
}

pub fn parse_message(input: &str) -> IResult<&str, Message> {
    let (input, tags) = opt(parse_tags)(input)?;
    let (input, prefix) = opt(parse_prefix)(input)?;
    let (input, command) = parse_param(input)?;
    let (input, params) = opt(parse_params)(input)?;
    let params = params.unwrap_or(Vec::new());

    // If there are any spaces leftover they had to be after a normal param, so
    // we consume those.
    let (input, _) = opt(is_a(" "))(input)?;

    // They *may* have included a line ending, so parse that.
    let (input, _) = opt(line_ending)(input)?;

    // Ensure we're at the end of the input.
    all_consuming(parse_empty_str)(input)?;

    Ok((
        input,
        Message {
            tags: tags.unwrap_or(BTreeMap::new()),
            prefix,
            command,
            params,
        },
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::iter::FromIterator;

    use super::Message;

    #[test]
    fn test_parse_tags() {
        assert_eq!(
            super::parse_tags("@a=b ").unwrap(),
            (
                "",
                BTreeMap::from_iter(vec![("a", "b".to_string())].iter().cloned())
            )
        );

        assert_eq!(
            super::parse_tags("@a=b;c=32;k;rt=ql7;z= ").unwrap(),
            (
                "",
                BTreeMap::from_iter(
                    vec![
                        ("a", "b".to_owned()),
                        ("c", "32".to_owned()),
                        ("k", "".to_owned()),
                        ("rt", "ql7".to_owned()),
                        ("z", "".to_owned()),
                    ]
                    .iter()
                    .cloned()
                )
            )
        );
    }

    #[test]
    fn test_parse_prefix() {
        assert_eq!(super::parse_prefix(":hello ").unwrap(), ("", "hello"));
        assert_eq!(
            super::parse_prefix(":hello!world@somewhere ").unwrap(),
            ("", "hello!world@somewhere")
        );
    }

    #[test]
    fn test_parse_param() {
        assert_eq!(super::parse_param("HELLO").unwrap(), ("", "HELLO"));
        assert_eq!(
            super::parse_param("HELLO:WORLD").unwrap(),
            ("", "HELLO:WORLD")
        );
        assert_eq!(
            super::parse_param("HELLO WORLD").unwrap(),
            (" WORLD", "HELLO")
        );
    }

    #[test]
    fn test_parse_trailing() {
        assert_eq!(super::parse_trailing(" :HELLO").unwrap(), ("", "HELLO"));
        assert_eq!(
            super::parse_trailing(" :HELLO:WORLD").unwrap(),
            ("", "HELLO:WORLD")
        );
        assert_eq!(
            super::parse_trailing(" :HELLO WORLD").unwrap(),
            ("", "HELLO WORLD")
        );

        assert_eq!(super::parse_trailing(" :#chan").unwrap(), ("", "#chan"));
    }

    #[test]
    fn test_parse_params() {
        assert_eq!(
            super::parse_params(" HELLO :WORLD").unwrap(),
            ("", vec!["HELLO", "WORLD"])
        );

        assert_eq!(
            super::parse_params(" HELLO :").unwrap(),
            ("", vec!["HELLO", ""])
        );

        assert_eq!(
            super::parse_params(" HELLO :#test").unwrap(),
            ("", vec!["HELLO", "#test"])
        );
    }

    #[test]
    fn test_escaped_char() {
        assert_eq!(super::parse_escaped_char(":").unwrap().1, ";");
    }

    #[test]
    fn test_parse_message() {
        assert_eq!(
            super::parse_message("HELLO").unwrap().1,
            Message {
                command: "HELLO",
                ..Message::default()
            }
        );

        assert_eq!(
            super::parse_message("HELLO WORLD").unwrap().1,
            Message {
                command: "HELLO",
                params: vec!["WORLD"],
                ..Message::default()
            }
        );

        assert_eq!(
            super::parse_message(":src JOIN :#chan").unwrap().1,
            Message {
                prefix: Some("src"),
                command: "JOIN",
                params: vec!["#chan"],
                ..Message::default()
            }
        );
    }
}
