use std::env::var_os;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::process::exit;
use std::str::Utf8Error;

pub(crate) struct EnvFailure {
    pub(crate) var: &'static str,
    pub(crate) err: EnvError,
}

pub(crate) enum EnvError {
    Missing,
    Empty,
    BadUnicode(Utf8Error),
}

pub(crate) fn noempty_utf8_env(var: &'static str) -> Result<Option<String>, EnvFailure> {
    match var_os(var) {
        None => Ok(None),
        Some(oss) => {
            if oss.is_empty() {
                Ok(None)
            } else {
                match String::from_utf8(oss.into_encoded_bytes()) {
                    Err(e) => Err(EnvFailure {
                        var,
                        err: EnvError::BadUnicode(e.utf8_error()),
                    }),
                    Ok(v) => Ok(Some(v)),
                }
            }
        }
    }
}

pub(crate) fn require_noempty_utf8_env(var: &'static str) -> Result<String, EnvFailure> {
    match var_os(var) {
        None => Err(EnvError::Missing),
        Some(oss) => {
            if oss.is_empty() {
                Err(EnvError::Empty)
            } else {
                String::from_utf8(oss.into_encoded_bytes())
                    .map_err(|err| EnvError::BadUnicode(err.utf8_error()))
            }
        }
    }
    .map_err(|err| EnvFailure { var, err })
}

impl Display for EnvFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Environment variable {} ", self.var)?;

        match self.err {
            EnvError::Missing => write!(f, "missing"),
            EnvError::Empty => write!(f, "is empty"),
            EnvError::BadUnicode(err) => write!(f, "is not valid UTF-8: {}", err),
        }
    }
}

pub(crate) fn exit_err<T, E: Display>(res: Result<T, E>) -> T {
    match res {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

pub(crate) fn log_err<T, E: Display>(res: Result<T, E>) {
    match res {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}
