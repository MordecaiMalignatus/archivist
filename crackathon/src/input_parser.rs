use anyhow::Result;
use anyhow::anyhow;
use itertools::Itertools;

#[derive(Default, Debug, Eq, PartialEq)]
pub struct Input {
    pub card_number: String,
    pub set_code: String,
    pub foil: bool,
    pub removal: bool,
}

pub fn parse_addition_input(mut input: String, provided_set_code: Option<String>) -> Result<Input> {
    let mut res = Input::default();
    let original_input = input.clone();

    let mut iter = input.chars().peekable();

    loop {
        match &iter.peek() {
            Some('-') => {
                res.removal = true;
                iter.next();
            }
            Some(c) => {
                // If a new input token starts with letters, it's a set code,
                // but it may contain numbers.
                if char::is_alphabetic(**c) {
                    res.set_code = iter
                        .peeking_take_while(|ic| !char::is_whitespace(*ic))
                        .collect();

                // if a new input starts with numbers it's a card number and may not contain letters.
                } else if char::is_digit(**c, 10) {
                    res.card_number = iter
                        .peeking_take_while(|ic| {
                            !char::is_whitespace(*ic) && char::is_digit(*ic, 10)
                        })
                        .collect();
                // we skip over whitespace.
                } else if char::is_whitespace(**c) {
                    iter.next();
                    continue;
                }
            }
            None => break,
        }
    }
    if input.ends_with("f") {
        res.foil = true;
        input = input
            .strip_suffix("f")
            .expect("input buffer should end with `f` if previously confirmed to end with `f`. ")
            .to_string();
    }

    match provided_set_code {
        Some(set) => {
            res.set_code = set;
            res.card_number = input.trim().to_string();
        }
        None => match input.split_once(' ') {
            Some((set, number)) => {
                res.set_code = set.to_string();
                res.card_number = number.to_string();
            }
            None => {
                return Err(anyhow!(
                    "Could not parse input '{original_input}' into setcode and number. Expecting input like 'dsk 12' or 'blb 51f'."
                ));
            }
        },
    }

    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

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
        let input = "dsk 1".to_string();
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
}
