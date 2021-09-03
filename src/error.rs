use rhai::plugin::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Error {
    E_INVIND,
    E_PROPNF,
    E_INVARG,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
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