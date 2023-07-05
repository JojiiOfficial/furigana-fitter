#[derive(Debug, PartialEq)]
pub enum FittingError {
    FuriganaDiffers,
    WordTooLong,
    WordTooShort,
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
