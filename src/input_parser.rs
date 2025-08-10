use anyhow::Result;
use anyhow::anyhow;

#[derive(Default, Debug, Eq, PartialEq)]
pub struct Input {
    pub card_number: String,
    pub set_code: String,
    pub foil: bool,
    pub removal: bool,
}

/// Parse the input given on the REPL. This is slightly tricky as this is
/// essentially a highly compact DSL. Previously this was a lot more freeform,
/// but now this is accepting exactly two words of input: the form
/// `<-><collector number><f> <setcode>`. This means that `12 dsk` is valid
/// input, but `dsk 12` is not.
///
/// - `-12 dsk` removes one of those copies from the  collection.
/// - `12f dsk` adds a foil version.
pub fn parse_addition_input(input: String, provided_set_code: Option<String>) -> Result<Input> {
    let mut res = Input::default();

    let word_clone = input.clone();
    let mut word_split = word_clone.split_ascii_whitespace();

    let mut number = word_split
        .next()
        .expect("a non-empty input line has at least one word?")
        .to_string();
    let set_code = word_split.next();

    if number.starts_with('-') {
        res.removal = true;
        number = number.strip_prefix('-').unwrap().to_string();
    }

    if number.ends_with('f') {
        res.foil = true;
        number = number.strip_suffix('f').unwrap().to_string();
    }

    res.card_number = number.parse()?;
    res.set_code = match set_code {
        Some(set) => match set.chars().all(char::is_alphanumeric) {
            true => set.to_string(),
            false => {
                return Err(anyhow!(
                    "Given set code (second word) was not alphanumeric: {set}"
                ));
            }
        },
        None => match provided_set_code {
            Some(set_code) => match set_code.chars().all(char::is_alphanumeric) {
                true => set_code.to_string(),
                false => {
                    return Err(anyhow!(
                        "Given set code (second word) was not alphanumeric: {set_code}"
                    ));
                }
            },
            None => {
                return Err(anyhow!(
                    "No setcode was specified on start-up, nor passed along in the input."
                ));
            }
        },
    };

    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO(sar): how do I document this behaviour in a readme?
    #[test]
    fn test_set_code_override() {
        let input = String::from("1 blb");
        let second_input = String::from("1");

        let expected = Input {
            card_number: "1".to_string(),
            set_code: "blb".to_string(),
            foil: false,
            removal: false,
        };
        let second_expected = Input {
            card_number: "1".to_string(),
            set_code: "dsk".to_string(),
            foil: false,
            removal: false,
        };

        let res = parse_addition_input(input, Some("dsk".to_string())).unwrap();
        let second_res = parse_addition_input(second_input, Some("dsk".to_string())).unwrap();

        assert_eq!(res, expected);
        assert_eq!(second_res, second_expected);
    }

    #[test]
    fn test_simple_input_provided_setcode() {
        let input = String::from("1");
        let expected = Input {
            card_number: "1".to_string(),
            set_code: "blb".to_string(),
            foil: false,
            removal: false,
        };

        let res = parse_addition_input(input, Some("blb".to_string())).unwrap();

        assert_eq!(res, expected)
    }

    #[test]
    fn test_simple_with_set() {
        let input = "1 dsk".to_string();
        let expected = Input {
            card_number: "1".to_string(),
            set_code: "dsk".to_string(),
            foil: false,
            removal: false,
        };

        let res = parse_addition_input(input, None).unwrap();
        assert_eq!(res, expected)
    }

    #[test]
    fn test_removal_input() {
        let input = "-2 dsk".to_string();
        let expected = Input {
            card_number: 2.to_string(),
            set_code: "dsk".to_string(),
            foil: false,
            removal: true,
        };
        let res = parse_addition_input(input, None).unwrap();
        assert_eq!(res, expected)
    }

    #[test]
    fn test_simple_foil_input() {
        let input = "1f".to_string();
        let expected = Input {
            card_number: "1".to_string(),
            set_code: "blb".to_string(),
            foil: true,
            removal: false,
        };
        let res = parse_addition_input(input, Some("blb".to_string())).unwrap();
        assert_eq!(res, expected)
    }

    #[test]
    fn test_only_allow_minus_on_card_number() {
        assert!(parse_addition_input("1 -dsk".to_string(), None).is_err());
        assert!(parse_addition_input("1-f".to_string(), None).is_err());
    }
}
