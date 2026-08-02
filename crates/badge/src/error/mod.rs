use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Message field not set on builder. Unable to make Format.")]
    MessageNotSet,
    #[error("The string `{0}` could not be parsed as a valid color option.")]
    BadColorString(String),
    #[error(
        "The string `{0}` could not be parsed as a valid style option.\
   Try one of (Flat, Plastic, FlatSquare, ForTheBadge, Social). Capitalization does not matter."
    )]
    BadStyleChoice(String),
    #[error("Font was not found when loading")]
    UnableToLoadFont,
}
