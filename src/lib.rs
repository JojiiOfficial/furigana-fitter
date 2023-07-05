mod exceptions;
pub mod fitting_error;

use std::vec;

use jp_utils::furi::{
    segment::{encode::FuriEncoder, kanji::Kanji, AsSegment},
    Furigana,
};

use crate::fitting_error::FittingError;

#[derive(Debug, PartialEq, Clone)]
enum SingleReadingSegment {
    Kana(String),
    Kanji { kanji: String, reading: String },
}

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
    let fitted = fit_furigana_onto_word(&broken_up, word)?;
    let result_furigana = convert_to_furigana(fitted);

    Ok(result_furigana)
}

fn break_up_furigana_into_singles(furigana: Furigana<&str>) -> Vec<SingleReadingSegment> {
    furigana
        .into_iter()
        .flat_map(|part| {
            part.reading_iter()
                .map(|reading| match reading.kanji() {
                    Some(kanji) => SingleReadingSegment::Kanji {
                        kanji: kanji.to_string(),
                        reading: reading.kana().to_string(),
                    },
                    None => SingleReadingSegment::Kana(reading.kana().to_string()),
                })
                .collect::<Vec<SingleReadingSegment>>()
        })
        .collect()
}

fn fit_furigana_onto_word<'a>(
    furigana: &Vec<SingleReadingSegment>,
    word: &str,
) -> Result<Vec<SingleReadingSegment>, FittingError> {
    let mut remaining_word = word.to_string();
    furigana
        .iter()
        .enumerate()
        .map(|(i, part)| {
            if remaining_word.is_empty() {
                return Err(FittingError::WordTooShort);
            }

            match part {
                SingleReadingSegment::Kanji { kanji, reading } => {
                    if remaining_word.starts_with(kanji) {
                        remaining_word =
                            remaining_word.chars().skip(kanji.chars().count()).collect();
                        return Ok(SingleReadingSegment::Kanji {
                            kanji: kanji.to_string(),
                            reading: reading.to_string(),
                        });
                    }

                    if remaining_word.starts_with(reading) {
                        remaining_word = remaining_word
                            .chars()
                            .skip(reading.chars().count())
                            .collect();
                        return Ok(SingleReadingSegment::Kana(reading.to_string()));
                    }

                    return Err(FittingError::FuriganaDiffers);
                }
                SingleReadingSegment::Kana(reading) => {
                    // Assumption: the kana parts can only change at the end of the word

                    let is_last = i == furigana.len() - 1;
                    if is_last {
                        let result = Ok(SingleReadingSegment::Kana(remaining_word.clone()));
                        remaining_word.clear();
                        return result;
                    }

                    if !remaining_word.starts_with(reading) {
                        return Err(FittingError::FuriganaDiffers);
                    }

                    remaining_word = remaining_word
                        .chars()
                        .skip(reading.chars().count())
                        .collect();

                    return Ok(SingleReadingSegment::Kana(reading.to_string()));
                }
            }
        })
        .collect::<Result<Vec<SingleReadingSegment>, FittingError>>()
        .and_then(|result| {
            if !remaining_word.is_empty() {
                return Err(FittingError::WordTooLong);
            }

            return Ok(result);
        })
}

fn convert_to_furigana(fitted: Vec<SingleReadingSegment>) -> String {
    let mut fitted_iter = fitted.iter();

    let mut result_string = String::new();
    let mut encoder = FuriEncoder::new(&mut result_string);

    let Some(last_segment) = fitted_iter.next() else {
        return "".to_string();
    };

    let mut last_kanji = match last_segment {
        SingleReadingSegment::Kana(kana) => {
            encoder.write_kana(kana);
            (None, vec![])
        }
        SingleReadingSegment::Kanji { kanji, reading } => {
            (Some(kanji.to_string()), vec![reading.to_owned()])
        }
    };

    last_kanji = fitted_iter.fold(last_kanji, |last_kanji, segment| match segment {
        SingleReadingSegment::Kana(kana) => match last_kanji {
            (Some(last_kanji), mut readings) => {
                encoder.write_kanji(Kanji::new(last_kanji, readings.as_slice()));
                encoder.write_kana(kana);
                readings.clear();
                (None, readings)
            }
            (None, empty_readings) => {
                encoder.write_kana(kana);
                (None, empty_readings)
            }
        },
        SingleReadingSegment::Kanji { kanji, reading } => match last_kanji {
            (Some(mut last_kanji), mut readings) => {
                last_kanji.push_str(kanji);
                readings.push(reading.to_owned());
                (Some(last_kanji), readings)
            }
            (None, mut readings) => {
                readings.push(reading.to_owned());
                (Some(kanji.to_string()), readings)
            }
        },
    });

    if let (Some(kanji), readings) = last_kanji {
        encoder.write_kanji(Kanji::new(kanji.to_string(), readings.as_slice()));
    }

    return result_string;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_input_output() {
        assert_eq!(
            fit_furigana("行く", "[行|い]く"),
            Ok("[行|い]く".to_owned())
        );
    }

    #[test]
    fn test_conjugated_verb() {
        assert_eq!(
            fit_furigana("行った", "[行|い]く"),
            Ok("[行|い]った".to_owned())
        );
    }

    #[test]
    fn test_exception_verb() {
        assert_eq!(
            fit_furigana("来る", "[来|く]る"),
            Ok("[来|く]る".to_owned())
        );
        assert_eq!(
            fit_furigana("来た", "[来|く]る"),
            Ok("[来|き]た".to_owned())
        );
        assert_eq!(
            fit_furigana("来い", "[来|く]る"),
            Ok("[来|こ]い".to_owned())
        );
        assert_eq!(
            fit_furigana("為る", "[為|す]る"),
            Ok("[為|す]る".to_owned())
        );
        assert_eq!(
            fit_furigana("為た", "[為|す]る"),
            Ok("[為|し]た".to_owned())
        );
        assert_eq!(
            fit_furigana("為て", "[為|す]る"),
            Ok("[為|し]て".to_owned())
        );
        assert_eq!(
            fit_furigana("為れる", "[為|す]る"),
            Ok("[為|さ]れる".to_owned())
        );
    }

    #[test]
    fn test_past_errors() {
        assert_eq!(
            fit_furigana("まき散らす", "まき[散|ち]らす"),
            Ok("まき[散|ち]らす".to_owned())
        );
        assert_eq!(
            fit_furigana("引っ掛かる", "[引|ひ]っ[掛|か]かる"),
            Ok("[引|ひ]っ[掛|か]かる".to_owned())
        );
    }

    #[test]
    fn test_partially_kana_word() {
        assert_eq!(
            fit_furigana("引っかかる", "[引|ひ]っ[掛|か]かる"),
            Ok("[引|ひ]っかかる".to_owned())
        );
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(fit_furigana("", ""), Ok("".to_owned()));
    }

    #[test]
    fn test_detailed_reading_breakdown() {
        assert_eq!(
            fit_furigana("音楽", "[音楽|おん|がく]"),
            Ok("[音楽|おん|がく]".to_owned())
        );
    }

    #[test]
    fn test_provided_word_too_short() {
        assert_eq!(
            fit_furigana("引", "[引|ひ]っ[掛|か]かる"),
            Err(FittingError::WordTooShort)
        );
    }

    #[test]
    fn test_kanji_reading_differs() {
        assert_eq!(
            fit_furigana("引っひかる", "[引|ひ]っ[掛|か]かる"),
            Err(FittingError::FuriganaDiffers)
        );
    }

    #[test]
    fn test_kana_part_differs() {
        assert_eq!(
            fit_furigana("引つ掛かる", "[引|ひ]っ[掛|か]かる"),
            Err(FittingError::FuriganaDiffers)
        );
    }

    #[test]
    fn test_word_too_long() {
        assert_eq!(
            fit_furigana("音楽あ", "[音楽|おん|がく]"),
            Err(FittingError::WordTooLong)
        );
    }
}
