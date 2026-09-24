#[derive(Debug, PartialEq)]
pub(crate) struct FuzzyItem<'a> {
    pub(crate) index: usize,
    pub(crate) label: &'a str,
    pub(crate) score: usize,
    pub(crate) positions: Vec<usize>,
}

fn fuzzy_score(text: &str, query: &str) -> (usize, Vec<usize>) {
    if query.is_empty() {
        return (1, vec![]);
    }
    let mut positions = vec![];

    let mut text_chars = text.chars().enumerate();

    for q in query.chars() {
        let mut found = None;

        for (idx, c) in text_chars.by_ref() {
            if c.to_lowercase().eq(q.to_lowercase()) {
                found = Some(idx);
                break;
            }
        }

        if let Some(idx) = found {
            positions.push(idx);
        } else {
            break;
        }
    }
    (score(&positions, query.chars().count()), positions)
}

fn score(v: &[usize], expected: usize) -> usize {
    if v.is_empty() || v.len() != expected {
        return 0;
    }

    let mut score = 1;

    for pair in v.windows(2) {
        let distance = pair[1].saturating_sub(pair[0]);
        if distance == 1 {
            score += 20;
        } else {
            score += 10usize.saturating_sub(distance);
        }
    }

    score
}

pub(crate) fn fuzzy_search<'a>(
    iter: impl Iterator<Item = &'a str>,
    query: &str,
) -> Vec<FuzzyItem<'a>> {
    let mut result = vec![];
    for (index, item) in iter.enumerate() {
        let (score, positions) = fuzzy_score(item, query);
        if score == 0 {
            continue;
        }
        result.push(FuzzyItem {
            index,
            label: item,
            score,
            positions,
        });
    }

    result.sort_by_key(|b| std::cmp::Reverse(b.score));

    result
}

#[cfg(test)]
mod fuzzy_score_tests {
    use super::*;

    #[test]
    fn empty_query_scores_zero_with_no_positions() {
        assert_eq!(fuzzy_score("Italia", ""), (1, vec![]));
    }

    #[test]
    fn contiguous_match_scores_higher_than_scattered() {
        assert_eq!(fuzzy_score("Francia", "fra"), (41, vec![0, 1, 2]));
    }

    #[test]
    fn ascii_case_insensitive_match_still_works() {
        assert_eq!(fuzzy_score("FRANCIA", "fra"), (41, vec![0, 1, 2]));
    }

    #[test]
    fn accented_uppercase_matches_accented_lowercase_query() {
        assert_eq!(fuzzy_score("PERÙ", "perù"), (61, vec![0, 1, 2, 3]));
    }

    #[test]
    fn multibyte_query_is_not_broken_by_byte_length_mismatch() {
        let (score, positions) = fuzzy_score("La città eterna", "città");
        assert_eq!(positions.len(), 5);
        assert!(score > 0);
    }

    #[test]
    fn scattered_subsequence_still_matches_with_lower_score() {
        assert_eq!(fuzzy_score("Barcellona", "bna"), (23, vec![0, 8, 9]));
    }

    #[test]
    fn out_of_order_characters_score_zero() {
        assert_eq!(fuzzy_score("ab", "ba").0, 0);
    }

    #[test]
    fn missing_character_scores_zero() {
        assert_eq!(fuzzy_score("Italia", "itax").0, 0);
    }

    #[test]
    fn query_longer_than_available_repeats_scores_zero() {
        assert_eq!(fuzzy_score("aa", "aaa").0, 0);
    }
}

#[cfg(test)]
mod fuzzy_search_tests {
    use super::*;

    #[test]
    fn empty_query_returns_everything_unfiltered_in_original_order() {
        let items = ["Italia", "Francia", "Germania"];

        assert_eq!(
            fuzzy_search(items.iter().copied(), ""),
            vec![
                FuzzyItem {
                    index: 0,
                    label: "Italia",
                    score: 1,
                    positions: vec![]
                },
                FuzzyItem {
                    index: 1,
                    label: "Francia",
                    score: 1,
                    positions: vec![]
                },
                FuzzyItem {
                    index: 2,
                    label: "Germania",
                    score: 1,
                    positions: vec![]
                },
            ]
        );
    }

    #[test]
    fn non_matching_items_are_dropped() {
        let items = ["Italia", "Francia", "Germania"];

        assert_eq!(
            fuzzy_search(items.iter().copied(), "ger"),
            vec![FuzzyItem {
                index: 2,
                label: "Germania",
                score: 41, // "ger": 3 char, 2 coppie a distanza 1 -> 1+20+20
                positions: vec![0, 1, 2],
            }]
        );
    }

    #[test]
    fn empty_items_returns_empty_result() {
        let items: [&str; 0] = [];
        assert_eq!(fuzzy_search(items.iter().copied(), "qualunque"), vec![]);
    }

    #[test]
    fn ties_preserve_original_item_order() {
        // Query di un solo carattere: `score()` con un'unica posizione
        // (nessuna coppia in windows(2)) vale sempre 1, indipendentemente
        // da dove sta il match -> Zebra e Apple pareggiano a score 1,
        // e l'ordine di partenza va preservato. Igloo non ha 'a', escluso.
        let items = ["Zebra", "Apple", "Igloo"];

        assert_eq!(
            fuzzy_search(items.iter().copied(), "a"),
            vec![
                FuzzyItem {
                    index: 0,
                    label: "Zebra",
                    score: 1,
                    positions: vec![4]
                },
                FuzzyItem {
                    index: 1,
                    label: "Apple",
                    score: 1,
                    positions: vec![0]
                },
            ]
        );
    }

    #[test]
    fn higher_score_ranks_first_even_out_of_original_order() {
        // "Abc" (match tutto contiguo: 1+20+20=41) deve precedere
        // "Xaybzc" (stesso testo ma sparso: 1+8+8=17) anche se
        // "Xaybzc" è il primo elemento della lista originale.
        let items = ["Xaybzc", "Abc"];

        assert_eq!(
            fuzzy_search(items.iter().copied(), "abc"),
            vec![
                FuzzyItem {
                    index: 1,
                    label: "Abc",
                    score: 41,
                    positions: vec![0, 1, 2]
                },
                FuzzyItem {
                    index: 0,
                    label: "Xaybzc",
                    score: 17,
                    positions: vec![1, 3, 5]
                },
            ]
        );
    }
}
