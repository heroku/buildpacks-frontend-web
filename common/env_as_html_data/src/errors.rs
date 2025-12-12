use std::fmt::{self, Debug};

#[derive(Debug)]
pub enum Error {
    ElementExpected(String),
    EncodeError(String),
    FileError(std::io::Error, String),
    NoHeadElementError,
    ParseError(String),
    SerializeError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ElementExpected(got_name) => {
                write!(
                    f,
                    "Env-as-HTML-Data expected an Element node, but got {got_name}"
                )
            }
            Error::EncodeError(error) => {
                write!(f, "Env-as-HTML-Data output encoding error, {error}")
            }
            Error::FileError(error, error_desc) => {
                write!(f, "Env-as-HTML-Data file error, {error_desc}, {error:#?}")
            }
            Error::NoHeadElementError => {
                write!(f, "Env-as-HTML-Data no head element found in document")
            }
            Error::ParseError(error) => {
                write!(f, "Env-as-HTML-Data document parsing error, {error}")
            }
            Error::SerializeError(error) => {
                write!(f, "Env-as-HTML-Data document serialization error, {error}")
            }
        }
    }
}
