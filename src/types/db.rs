use core::fmt;

pub enum CustomDbErr {
    NotFound,
    Unimplemented,
}

impl fmt::Display for CustomDbErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomDbErr::NotFound => write!(f, "NotFound"),
            CustomDbErr::Unimplemented => write!(f, "Unimplemented"),
        }
    }
}

impl std::str::FromStr for CustomDbErr {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NotFound" => Ok(CustomDbErr::NotFound),
            _ => Ok(CustomDbErr::Unimplemented),
        }
    }
}
