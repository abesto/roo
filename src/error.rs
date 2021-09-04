use rhai::plugin::*;
use strum_macros::{Display, EnumIter, EnumString};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, EnumString, Display, EnumIter)]
pub enum Error {
    E_NONE,
    E_TYPE,
    E_DIV,
    E_PERM,
    E_PROPNF,
    E_VERBNF,
    E_VARNF,
    E_INVIND,
    E_RECMOVE,
    E_MAXREC,
    E_RANGE,
    E_ARGS,
    E_NACC,
    E_INVARG,
    E_QUOTA,
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
        Box::new(EvalAltResult::ErrorRuntime(
            Dynamic::from(e).into_shared(),
            rhai::Position::NONE,
        ))
    }
}

impl std::error::Error for Error {}
