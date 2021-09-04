use rhai::plugin::*;
use strum::EnumMessage;
use strum_macros::{Display, EnumIter, EnumString, EnumMessage};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, EnumString, Display, EnumIter, EnumMessage)]
pub enum Error {
    #[strum(message = "No error")]
    E_NONE,
    #[strum(message = "Type mismatch")]
    E_TYPE,
    #[strum(message = "Division by zero")]
    E_DIV,
    #[strum(message = "Permission denied")]
    E_PERM,
    #[strum(message = "Property not found")]
    E_PROPNF,
    #[strum(message = "Verb not found")]
    E_VERBNF,
    #[strum(message = "Variable not found")]
    E_VARNF,
    #[strum(message = "Invalid indirection")]
    E_INVIND,
    #[strum(message = "Recursive move")]
    E_RECMOVE,
    #[strum(message = "Too many verb calls")]
    E_MAXREC,
    #[strum(message = "Range error")]
    E_RANGE,
    #[strum(message = "Incorrect number of arguments")]
    E_ARGS,
    #[strum(message = "Move refused by destination")]
    E_NACC,
    #[strum(message = "Invalid argument")]
    E_INVARG,
    #[strum(message = "Resource limit exceeded")]
    E_QUOTA,
    #[strum(message = "Floating-point arithmetic error")]
    E_FLOAT,
}

pub type RhaiError = Box<EvalAltResult>;
pub type RhaiResult<T> = Result<T, RhaiError>;

macro_rules! bail {
    ($e:expr) => {
        return Err($e.into())
    };
}

impl From<Error> for RhaiError {
    fn from(e: Error) -> Self {
        let mut map = rhai::Map::new();
        map.insert("error_marker".into(), true.into());
        map.insert("code".into(), format!("{}", e).into());
        map.insert("message".into(), e.get_message().unwrap().into());
        Box::new(EvalAltResult::ErrorRuntime(
            Dynamic::from(map),
            rhai::Position::NONE,
        ))
    }
}

impl std::error::Error for Error {}
