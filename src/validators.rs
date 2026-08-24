//! A handful of common, general-purpose [`Validator`]s for use with
//! [`crate::builder`]'s `.validator(...)`.
//!
//! Every function here takes the error message to show when the check
//! fails, and returns a `Validator` ready to pass to `.validator(...)`.
//! None of them special-case the empty string on their own — inside a
//! `Field`, the required check already decides whether an empty value is
//! acceptable before any of these run, so they never see one there. Calling
//! them directly, outside of a `Field`, is a different story: see each
//! function's notes below.

use std::str::FromStr;

use crate::field::Validator;

/// Rejects an empty value. This is what `.required(message)` uses
/// internally (see [`crate::builder`]) — you'll rarely call it directly,
/// but it's a plain `Validator` like the rest, so nothing stops you from
/// composing it yourself if you need to.
pub fn required(message: String) -> Validator {
    Box::new(move |value: &str| {
        (!value.is_empty())
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}

/// Rejects a value shorter than `len` characters. The bound is inclusive:
/// `min_length(5, ..)` accepts a value that is exactly 5 characters long.
///
/// Counts Unicode characters, not bytes — `"città"` is 5 characters even
/// though it's 6 bytes in UTF-8. Passes on an empty value when called
/// directly (see the module-level note above).
pub fn min_length(len: usize, message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().count() >= len)
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}

/// Rejects a value longer than `len` characters. The bound is inclusive:
/// `max_length(10, ..)` accepts a value that is exactly 10 characters long.
///
/// Counts Unicode characters, not bytes, same as [`min_length`].
pub fn max_length(len: usize, message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().count() <= len)
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}

/// Rejects a value that isn't made up entirely of ASCII digits (`0`–`9`).
///
/// No sign and no decimal point are accepted — `"-5"` and `"3.14"` are both
/// rejected. For a value that must actually parse as a number (including
/// negative or floating-point ones), use [`parsable`] instead.
pub fn is_numeric(message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().all(|c| c.is_ascii_digit()))
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}

/// Rejects a value that isn't made up entirely of letters.
///
/// Unicode-aware: accented letters such as `à` count as letters, this is
/// not limited to ASCII.
pub fn alphabetic(message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().all(|c| c.is_alphabetic()))
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}

/// Rejects a value that isn't made up entirely of letters and digits.
///
/// Unicode-aware for letters, same as [`alphabetic`].
pub fn alphanumeric(message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().all(|c| c.is_alphanumeric()))
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}

/// Rejects a value that contains any whitespace — spaces, tabs, newlines,
/// or any other character `char::is_whitespace` considers whitespace.
pub fn no_whitespace(message: String) -> Validator {
    Box::new(move |value: &str| {
        (value.chars().all(|c| !c.is_whitespace()))
            .then_some(())
            .ok_or_else(|| message.clone())
    })
}

/// Rejects a value that doesn't parse as a `T` via `T: FromStr`.
///
/// Unlike the other validators in this module, `T` only appears in the
/// function body, not in `Validator`'s return type, so the compiler can't
/// infer it — the turbofish is required:
///
/// ```text
/// validators::parsable::<i32>("not a valid number".to_owned())
/// ```
///
/// This isn't limited to numbers: any type with a `FromStr` implementation
/// works, including ones from other crates. For example, `chrono::NaiveDate`
/// implements `FromStr` for the ISO `YYYY-MM-DD` format, so
/// `parsable::<chrono::NaiveDate>(..)` gives a correct, leap-year-aware date
/// validator without `ratiform` itself depending on `chrono` — that
/// dependency, if you want it, lives in your own `Cargo.toml`.
///
/// Note that, unlike the shape-based validators above, an empty value is
/// never valid here: `"".parse::<i32>()` fails, same as any other malformed
/// input.
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
