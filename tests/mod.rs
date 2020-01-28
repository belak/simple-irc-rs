use std::borrow::Cow;
use std::collections::BTreeMap;
use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use simple_irc::Message;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestAtoms {
    #[serde(default)]
    tags: BTreeMap<String, String>,
    source: Option<String>,
    verb: String,
    #[serde(default)]
    params: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MsgSplitTest {
    input: String,
    atoms: TestAtoms,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MsgSplitTests {
    tests: Vec<MsgSplitTest>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MsgJoinTest {
    desc: String,
    matches: Vec<String>,
    atoms: TestAtoms,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MsgJoinTests {
    tests: Vec<MsgJoinTest>,
}

#[test]
fn test_msg_split() {
    let msg_split_test_data = include_str!("external/parser-tests/tests/msg-split.yaml");
    let tests = serde_yaml::from_str::<MsgSplitTests>(msg_split_test_data).unwrap();

    for test in tests.tests {
        println!("Trying {}", &test.input);

        let res = Message::try_from(&test.input[..]);

        // Ensure all messages parse into something
        assert!(
            !res.is_err(),
            "msg failed: \"{}\", err {}",
            &test.input,
            res.unwrap_err()
        );

        let msg = res.unwrap();

        let mut msg_tags = msg.tags.clone();

        // Loop through all the test tags and make sure they were there.
        for (key, value) in test.atoms.tags {
            assert_eq!(
                value,
                msg_tags.remove(key.as_str()).unwrap(),
                "Mismatched value for key {}",
                key.as_str()
            );

            // Remove any keys we found from msg_tags so we can ensure there
            // were no leftovers later.
            msg_tags.remove(key.as_str());
        }

        // If there are any tags left over in msg_tags, this is an error.
        for (key, value) in msg_tags {
            assert!(false, "Extra value {} for key {}", value, key);
        }

        let prefix = test.atoms.source.as_deref();

        assert_eq!(
            prefix, msg.prefix,
            "msg prefix mismatch: expected \"{:?}\" got \"{:?}\"",
            prefix, msg.prefix,
        );

        assert_eq!(
            test.atoms.verb, msg.command,
            "msg command mismatch: expected \"{}\" got \"{}\"",
            test.atoms.verb, msg.command,
        );

        let params: Vec<&str> = test.atoms.params.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            params, msg.params,
            "msg params mismatch: expected \"{:?}\" got \"{:?}\"",
            params, msg.params,
        );
    }
}

#[test]
fn test_msg_join() {
    let msg_split_test_data = include_str!("external/parser-tests/tests/msg-join.yaml");
    let tests = serde_yaml::from_str::<MsgJoinTests>(msg_split_test_data).unwrap();

    for test in tests.tests {
        let mut tags = BTreeMap::new();

        for (k, v) in test.atoms.tags.iter() {
            tags.insert(k.as_str(), Cow::Borrowed(v.as_str()));
        }

        let msg = Message {
            tags,
            prefix: test.atoms.source.as_deref(),
            command: test.atoms.verb.as_str(),
            params: test.atoms.params.iter().map(|s| s.as_str()).collect(),
        };

        let out = format!("{}", msg);

        assert!(
            test.matches.contains(&out.to_string()),
            "expected one of: {:?}, got: {:?}",
            test.matches,
            out
        );
    }
}
