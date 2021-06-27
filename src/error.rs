use mlua::ToLua;

#[allow(non_camel_case_types, dead_code)]
#[derive(Debug)]
pub enum ErrorCode {
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

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl ErrorCode {
    #[must_use]
    pub fn make<S>(self, message: S) -> Error
    where
        S: ToString,
    {
        Error::new(self, message.to_string())
    }
}

#[derive(Debug)]
pub struct Error {
    code: ErrorCode,
    message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl Error {
    #[must_use]
    pub fn new(code: ErrorCode, message: String) -> Self {
        Error { code, message }
    }
}

impl<'lua> ToLua<'lua> for &Error {
    fn to_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let ctor: mlua::Function = lua.globals().get(self.code.to_string())?;
        ctor.call(self.message.clone())
    }
}
