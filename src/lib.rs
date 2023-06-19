use jp_utils::furigana::{
    segment::{Segment, SegmentRef},
    seq::FuriSequence,
    Furigana,
};
use tinyvec::tiny_vec;

#[derive(Debug, PartialEq)]
pub enum FittingError {
    FuriganaDiffers,
    WordTooLong,
    WordTooShort,
}

#[derive(Debug, PartialEq)]
enum SingleReadingSegment {
    Kana(String),
    Kanji { kanji: String, reading: String },
}

impl std::fmt::Display for FittingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FittingError::FuriganaDiffers => {
                write!(f, "The furigana differs from the provided word")
            }
            FittingError::WordTooLong => {
                write!(f, "The word is too long to fit the furigana")
            }
            FittingError::WordTooShort => {
                write!(f, "The word is too short to fit the furigana")
            }
        }
    }
}

pub fn fit_furigana(word: &str, raw_furigana: &str) -> Result<String, FittingError> {
    let parsed_furigana = Furigana(raw_furigana);
    let parsed_furigana_string = parsed_furigana.kanji_str();

    if parsed_furigana_string == "来る" {
        return Ok(handle_kuru(word));
    }

    if parsed_furigana_string == "為る" {
        return Ok(handle_suru(word));
    }

    let broken_up = break_up_furigana_into_singles(parsed_furigana);
    let fitted = fit_furigana_onto_word(&broken_up, word)?;
    let result_furigana = convert_to_furigana(fitted);

    Ok(result_furigana)
}

fn break_up_furigana_into_singles(furigana: Furigana<&str>) -> Vec<SingleReadingSegment> {
    furigana
        .into_iter()
        .flat_map(|part| match part {
            SegmentRef::Kana(reading) => vec![SingleReadingSegment::Kana(reading.to_string())],
            SegmentRef::Kanji { kanji, readings } => {
                if let Some(first_reading) = readings.first() {
                    return vec![SingleReadingSegment::Kanji {
                        kanji: kanji.to_string(),
                        reading: first_reading.to_string(),
                    }];
                }

                kanji
                    .chars()
                    .zip(readings.iter())
                    .map(|(kanji, reading)| SingleReadingSegment::Kanji {
                        kanji: kanji.to_string(),
                        reading: reading.to_string(),
                    })
                    .collect()
            }
        })
        .collect()
}

fn fit_furigana_onto_word<'a>(
    furigana: &Vec<SingleReadingSegment>,
    word: &str,
) -> Result<Vec<SingleReadingSegment>, FittingError> {
    let mut remaining_word = word.to_string();
    println!("Remaining word: {}", remaining_word);
    furigana
        .iter()
        .enumerate()
        .map(|(i, part)| {
            println!("Part: {:?} Remaining: {:?}", part, remaining_word);
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

    let Some(first_fitted) = fitted_iter.next() else {
        return "".to_string();
    };

    let first = match first_fitted {
        SingleReadingSegment::Kana(reading) => Segment::Kana(reading.to_string()),
        SingleReadingSegment::Kanji { kanji, reading } => Segment::Kanji {
            kanji: kanji.to_string(),
            readings: tiny_vec![reading.to_string()],
        },
    };

    let mut result_segments = vec![first];

    while let Some(segment) = fitted_iter.next() {
        // There's always something in the vector
        let last_segment = result_segments.last_mut().unwrap();

        match segment {
            SingleReadingSegment::Kana(reading) => match last_segment {
                Segment::Kana(last_reading) => {
                    last_reading.push_str(reading);
                }
                Segment::Kanji { .. } => {
                    result_segments.push(Segment::Kana(reading.to_string()));
                }
            },
            SingleReadingSegment::Kanji { kanji, reading } => match last_segment {
                Segment::Kana(_) => {
                    result_segments.push(Segment::Kanji {
                        kanji: kanji.to_string(),
                        readings: tiny_vec![reading.to_string()],
                    });
                }
                Segment::Kanji {
                    kanji: last_kanji,
                    readings: last_readings,
                } => {
                    last_kanji.push_str(kanji);
                    last_readings.push(reading.to_string());
                }
            },
        }
    }

    FuriSequence::from(result_segments).to_string()
}

fn handle_kuru(word: &str) -> String {
    if word == "来る" {
        return "[来|く]る".to_string();
    }

    let tail = word.chars().skip(1).collect::<String>();

    let is_formal = word.starts_with("来ま");
    let is_te_form = word.starts_with("来て");
    let is_ta_form = word.starts_with("来た");
    if is_formal || is_te_form || is_ta_form {
        return format!("[来|き]{}", tail).to_string();
    }

    return format!("[来|こ]{}", tail).to_string();
}

fn handle_suru(word: &str) -> String {
    if word == "為る" {
        return "[為|す]る".to_string();
    }
    let tail = word.chars().skip(1).collect::<String>();

    let is_formal = word.starts_with("為ま");
    let is_te_form = word.starts_with("為て");
    let is_ta_form = word.starts_with("為た");
    let is_imperative = word.starts_with("為ろ");
    if is_formal || is_te_form || is_ta_form || is_imperative {
        return format!("[為|し]{}", tail).to_string();
    }

    return format!("[為|さ]{}", tail).to_string();
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
