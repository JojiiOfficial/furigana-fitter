use furigana_fitter::{fitting_error::FittingError, fit_furigana};

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
