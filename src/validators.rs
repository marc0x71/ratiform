use std::str::FromStr;

use crate::field::Validator;

pub fn required(message: String) -> Validator {
    Box::new(move |value: &str| {
        (!value.is_empty())
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}
pub fn min_length(len: usize, message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().count() >= len)
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}
pub fn max_length(len: usize, message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().count() <= len)
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}
pub fn is_numeric(message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().all(|c| c.is_ascii_digit()))
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}
pub fn alphabetic(message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().all(|c| c.is_alphabetic()))
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}
pub fn alphanumeric(message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().all(|c| c.is_alphanumeric()))
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}
pub fn no_whitespace(message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().all(|c| !c.is_whitespace()))
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}
pub fn parsable<T: FromStr>(message: String) -> Validator {
    Box::new(move |value: &str| value.parse::<T>().map(|_| ()).map_err(|_| message.clone()))
}

#[cfg(test)]
mod validators_tests {
    use super::*;

    // ---------- min_length ----------

    #[test]
    fn min_length_accepts_value_at_exactly_the_limit() {
        let validator = min_length(5, "too short".to_owned());
        assert_eq!(validator("ciaoo"), Ok(()));
    }

    #[test]
    fn min_length_rejects_value_one_char_below_the_limit() {
        let validator = min_length(5, "too short".to_owned());
        assert_eq!(validator("ciao"), Err("too short".to_owned()));
    }

    #[test]
    fn min_length_accepts_value_above_the_limit() {
        let validator = min_length(5, "too short".to_owned());
        assert_eq!(validator("ciaociao"), Ok(()));
    }

    #[test]
    fn min_length_counts_characters_not_bytes() {
        // "città" = 5 caratteri, 6 byte (la 'à' occupa 2 byte in UTF-8).
        let validator = min_length(5, "too short".to_owned());
        assert_eq!(validator("città"), Ok(()));
    }

    #[test]
    fn min_length_of_zero_accepts_empty_value() {
        let validator = min_length(0, "too short".to_owned());
        assert_eq!(validator(""), Ok(()));
    }

    #[test]
    fn min_length_validator_can_be_called_more_than_once() {
        // Verifica che il messaggio catturato via .clone() non venga
        // "consumato" dopo la prima chiamata (Validator è Fn, non FnOnce).
        let validator = min_length(5, "too short".to_owned());
        assert_eq!(validator("ci"), Err("too short".to_owned()));
        assert_eq!(validator("ci"), Err("too short".to_owned()));
    }

    // ---------- max_length ----------

    #[test]
    fn max_length_accepts_value_at_exactly_the_limit() {
        let validator = max_length(10, "too long".to_owned());
        assert_eq!(validator("ciaociaoci"), Ok(()));
    }

    #[test]
    fn max_length_rejects_value_one_char_above_the_limit() {
        let validator = max_length(10, "too long".to_owned());
        assert_eq!(validator("ciaociaocia"), Err("too long".to_owned()));
    }

    #[test]
    fn max_length_accepts_value_below_the_limit() {
        let validator = max_length(10, "too long".to_owned());
        assert_eq!(validator("ciao"), Ok(()));
    }

    #[test]
    fn max_length_counts_characters_not_bytes() {
        // "città" = 5 caratteri, 6 byte: con max_length(5) deve passare
        // contando i caratteri, anche se i byte sarebbero 6.
        let validator = max_length(5, "too long".to_owned());
        assert_eq!(validator("città"), Ok(()));
    }

    #[test]
    fn max_length_accepts_empty_value() {
        let validator = max_length(10, "too long".to_owned());
        assert_eq!(validator(""), Ok(()));
    }

    // ---------- is_numeric ----------

    #[test]
    fn is_numeric_accepts_only_digits() {
        let validator = is_numeric("not numeric".to_owned());
        assert_eq!(validator("12345"), Ok(()));
    }

    #[test]
    fn is_numeric_rejects_letters_mixed_in() {
        let validator = is_numeric("not numeric".to_owned());
        assert_eq!(validator("123a5"), Err("not numeric".to_owned()));
    }

    #[test]
    fn is_numeric_rejects_a_minus_sign() {
        // Documenta lo scope della funzione: non gestisce i numeri negativi,
        // '-' non è un ascii digit.
        let validator = is_numeric("not numeric".to_owned());
        assert_eq!(validator("-123"), Err("not numeric".to_owned()));
    }

    #[test]
    fn is_numeric_rejects_non_ascii_digits() {
        // '１' è il carattere Unicode fullwidth per '1' (U+FF11):
        // non è un ascii digit, quindi deve essere rifiutato.
        let validator = is_numeric("not numeric".to_owned());
        assert_eq!(validator("１"), Err("not numeric".to_owned()));
    }

    #[test]
    fn is_numeric_accepts_empty_value() {
        // Comportamento documentato: .all() su un iteratore vuoto è
        // vacuamente vero. In pratica, dentro un Field, required()/optional()
        // intercettano il valore vuoto prima che questo validator giri.
        let validator = is_numeric("not numeric".to_owned());
        assert_eq!(validator(""), Ok(()));
    }

    // ---------- alphabetic ----------

    #[test]
    fn alphabetic_accepts_only_letters() {
        let validator = alphabetic("not alphabetic".to_owned());
        assert_eq!(validator("ciao"), Ok(()));
    }

    #[test]
    fn alphabetic_accepts_accented_letters() {
        // is_alphabetic() è Unicode-aware, non solo ASCII.
        let validator = alphabetic("not alphabetic".to_owned());
        assert_eq!(validator("città"), Ok(()));
    }

    #[test]
    fn alphabetic_rejects_digits() {
        let validator = alphabetic("not alphabetic".to_owned());
        assert_eq!(validator("ciao1"), Err("not alphabetic".to_owned()));
    }

    #[test]
    fn alphabetic_rejects_spaces() {
        let validator = alphabetic("not alphabetic".to_owned());
        assert_eq!(validator("ciao mondo"), Err("not alphabetic".to_owned()));
    }

    #[test]
    fn alphabetic_accepts_empty_value() {
        let validator = alphabetic("not alphabetic".to_owned());
        assert_eq!(validator(""), Ok(()));
    }

    // ---------- alphanumeric ----------

    #[test]
    fn alphanumeric_accepts_letters_and_digits_mixed() {
        let validator = alphanumeric("not alphanumeric".to_owned());
        assert_eq!(validator("ciao123"), Ok(()));
    }

    #[test]
    fn alphanumeric_accepts_accented_letters_with_digits() {
        let validator = alphanumeric("not alphanumeric".to_owned());
        assert_eq!(validator("città5"), Ok(()));
    }

    #[test]
    fn alphanumeric_rejects_spaces() {
        let validator = alphanumeric("not alphanumeric".to_owned());
        assert_eq!(validator("ciao 123"), Err("not alphanumeric".to_owned()));
    }

    #[test]
    fn alphanumeric_rejects_punctuation() {
        let validator = alphanumeric("not alphanumeric".to_owned());
        assert_eq!(validator("ciao!"), Err("not alphanumeric".to_owned()));
    }

    #[test]
    fn alphanumeric_accepts_empty_value() {
        let validator = alphanumeric("not alphanumeric".to_owned());
        assert_eq!(validator(""), Ok(()));
    }

    // ---------- no_whitespace ----------

    #[test]
    fn no_whitespace_accepts_value_without_spaces() {
        let validator = no_whitespace("contains whitespace".to_owned());
        assert_eq!(validator("ciaomondo"), Ok(()));
    }

    #[test]
    fn no_whitespace_rejects_a_space() {
        let validator = no_whitespace("contains whitespace".to_owned());
        assert_eq!(
            validator("ciao mondo"),
            Err("contains whitespace".to_owned())
        );
    }

    #[test]
    fn no_whitespace_rejects_a_tab() {
        let validator = no_whitespace("contains whitespace".to_owned());
        assert_eq!(
            validator("ciao\tmondo"),
            Err("contains whitespace".to_owned())
        );
    }

    #[test]
    fn no_whitespace_rejects_a_newline() {
        let validator = no_whitespace("contains whitespace".to_owned());
        assert_eq!(
            validator("ciao\nmondo"),
            Err("contains whitespace".to_owned())
        );
    }

    #[test]
    fn no_whitespace_accepts_empty_value() {
        let validator = no_whitespace("contains whitespace".to_owned());
        assert_eq!(validator(""), Ok(()));
    }

    // ---------- parseble ----------

    #[test]
    fn parseble_accepts_a_valid_integer() {
        let validator = parsable::<i32>("not a valid number".to_owned());
        assert_eq!(validator("42"), Ok(()));
    }

    #[test]
    fn parseble_rejects_non_numeric_text_as_integer() {
        let validator = parsable::<i32>("not a valid number".to_owned());
        assert_eq!(validator("abc"), Err("not a valid number".to_owned()));
    }

    #[test]
    fn parseble_rejects_an_integer_out_of_range() {
        // Sintatticamente "sembra" un numero, ma non ci sta in un i32:
        // un caso che is_numeric non distinguerebbe, parseble sì.
        let validator = parsable::<i32>("not a valid number".to_owned());
        assert_eq!(
            validator("99999999999999999999"),
            Err("not a valid number".to_owned())
        );
    }

    #[test]
    fn parseble_accepts_a_valid_float() {
        let validator = parsable::<f64>("not a valid number".to_owned());
        assert_eq!(validator("3.14"), Ok(()));
    }

    #[test]
    fn parseble_rejects_malformed_float() {
        let validator = parsable::<f64>("not a valid number".to_owned());
        assert_eq!(validator("3.14.15"), Err("not a valid number".to_owned()));
    }

    #[test]
    fn parseble_rejects_empty_value_as_integer() {
        // A differenza di is_numeric, qui la stringa vuota NON è valida:
        // "".parse::<i32>() fallisce sempre. Comportamento opposto e voluto,
        // non un'incoerenza tra le due funzioni.
        let validator = parsable::<i32>("not a valid number".to_owned());
        assert_eq!(validator(""), Err("not a valid number".to_owned()));
    }

    #[test]
    fn parseble_validator_can_be_called_more_than_once() {
        let validator = parsable::<i32>("not a valid number".to_owned());
        assert_eq!(validator("abc"), Err("not a valid number".to_owned()));
        assert_eq!(validator("abc"), Err("not a valid number".to_owned()));
    }
}
