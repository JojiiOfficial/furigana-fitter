mod exceptions;
pub mod fitting_error;

use std::vec;

use jp_utils::{
    furi::{
        segment::{encode::FuriEncoder, kanji::KanjiRef, AsSegment},
        Furigana,
    },
    reading::Reading,
};

use crate::fitting_error::FittingError;

pub fn fit_furigana(word: &str, raw_furigana: &str) -> Result<String, FittingError> {
    let parsed_furigana = Furigana(raw_furigana);
    let parsed_furigana_string = parsed_furigana.kanji_str();

    if parsed_furigana_string == "来る" {
        return Ok(exceptions::handle_kuru(word));
    }

    if parsed_furigana_string == "為る" {
        return Ok(exceptions::handle_suru(word));
    }

    let broken_up = break_up_furigana_into_singles(parsed_furigana);
    let fitted = fit_furigana_onto_word(broken_up, word)?;
    let result_furigana = convert_to_furigana(fitted);

    Ok(result_furigana)
}

fn break_up_furigana_into_singles(furigana: Furigana<&str>) -> Vec<Reading> {
    furigana
        .into_iter()
        .flat_map(|part| part.reading_iter().collect::<Vec<Reading>>())
        .collect()
}

fn fit_furigana_onto_word(
    furigana: Vec<Reading>,
    mut word: &str,
) -> Result<Vec<Reading>, FittingError> {
    furigana
        .iter()
        .enumerate()
        .map(|(i, part)| {
            if word.is_empty() {
                return Err(FittingError::WordTooShort);
            }

            match (part.kanji(), part.kana()) {
                (Some(kanji), kana) => {
                    if let Some(remaining) = word.strip_prefix(kanji) {
                        word = remaining;
                        return Ok(part.clone());
                    }

                    if let Some(remaining) = word.strip_prefix(kana) {
                        word = remaining;
                        return Ok(Reading::new(kana.to_string()));
                    }

                    Err(FittingError::FuriganaDiffers)
                }
                (None, kana) => {
                    // Assumption: the kana parts can only change at the end of the word

                    let is_last = i == furigana.len() - 1;
                    if is_last {
                        let result = Ok(Reading::new(word.to_string()));
                        word = "";
                        return result;
                    }

                    if let Some(remaining) = word.strip_prefix(kana) {
                        word = remaining;
                        return Ok(part.clone());
                    }

                    Err(FittingError::FuriganaDiffers)
                }
            }
        })
        .collect::<Result<Vec<Reading>, FittingError>>()
        .and_then(|result| {
            if !word.is_empty() {
                return Err(FittingError::WordTooLong);
            }

            Ok(result)
        })
}

fn convert_to_furigana(fitted: Vec<Reading>) -> String {
    let mut fitted_iter = fitted.iter();

    let mut result_string = String::new();
    let mut encoder = FuriEncoder::new(&mut result_string);

    let Some(last_segment) = fitted_iter.next() else {
        return result_string;
    };

    let mut last_kanji = match (last_segment.kanji(), last_segment.kana()) {
        (None, kana) => {
            encoder.write_kana(kana);
            (None, vec![])
        }
        (Some(kanji), kana) => (Some(kanji.to_string()), vec![kana]),
    };

    last_kanji = fitted_iter.fold(last_kanji, |last_kanji, segment| {
        match (segment.kanji(), segment.kana()) {
            (None, kana) => match last_kanji {
                (Some(last_kanji), mut readings) => {
                    encoder.write_kanji(KanjiRef::new(&last_kanji, readings.as_slice()));
                    encoder.write_kana(kana);
                    readings.clear();
                    (None, readings)
                }
                _ => {
                    encoder.write_kana(kana);
                    last_kanji
                }
            },
            (Some(kanji), kana) => match last_kanji {
                (Some(mut last_kanji), mut readings) => {
                    last_kanji.push_str(kanji);
                    readings.push(kana);
                    (Some(last_kanji), readings)
                }
                (None, mut readings) => {
                    readings.push(kana);
                    (Some(kanji.to_string()), readings)
                }
            },
        }
    });

    if let (Some(kanji), readings) = last_kanji {
        encoder.write_kanji(KanjiRef::new(&kanji, readings.as_slice()));
    }

    result_string
}
