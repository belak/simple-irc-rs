use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use simple_irc::parse_message;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MsgSplitTestAtoms {
    #[serde(default)]
    tags: BTreeMap<String, String>,
    source: Option<String>,
    verb: String,
    params: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MsgSplitTest {
    input: String,
    atoms: MsgSplitTestAtoms,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct MsgSplitTests {
    tests: Vec<MsgSplitTest>,
}

#[test]
fn test_msg_split() {
    let msg_split_test_data = include_str!("external/parser-tests/tests/msg-split.yaml");
    let tests = serde_yaml::from_str::<MsgSplitTests>(msg_split_test_data).unwrap();

    for test in tests.tests {
        println!("Trying {}", &test.input);

        let res = parse_message(&test.input);

        // Ensure all messages parse into something
        assert!(
            !res.is_err(),
            "msg failed: \"{}\", err {}",
            &test.input,
            res.unwrap_err()
        );

        let msg = res.unwrap();

        let mut test_tags = test.atoms.tags.clone();
        let mut msg_tags = msg.1.tags.clone();

        println!("{:?} {:?}", test_tags, msg_tags);

        // Loop through all the test tags and make sure they were there.
        for (key, value) in test_tags.clone() {
            assert_eq!(&value, msg_tags.get(key.as_str()).unwrap(), "Mismatched value for key {}", key.as_str());
            test_tags.remove(&key);
            msg_tags.remove(&key[..]);
        }

        // If there are any tags left over in msg_tags, this is an error.
        for (key, value) in msg_tags.clone() {
            assert!(false, "Extra value {} for key {}", value, key);
        }

        let prefix = if let Some(p) = &test.atoms.source {
            Some(p.as_str())
        } else {
            None
        };

        assert_eq!(
            prefix, msg.1.prefix,
            "msg prefix mismatch: expected \"{:?}\" got \"{:?}\"",
            prefix, msg.1.prefix,
        );

        assert_eq!(
            &test.atoms.verb, msg.1.command,
            "msg command mismatch: expected \"{}\" got \"{}\"",
            &test.atoms.verb, msg.1.command,
        );

        if let Some(params) = &test.atoms.params {
            let params: Vec<&str> = params.iter().map(|s| &s[..]).collect();
            assert_eq!(
                params, msg.1.params,
                "msg params mismatch: expected \"{:?}\" got \"{:?}\"",
                params, msg.1.params,
            );
        } else {
            assert!(
                msg.1.params.len() == 0,
                "msg params mismatch: expected no params got \"{:?}\"",
                msg.1.params
            );
        }
    }
}
