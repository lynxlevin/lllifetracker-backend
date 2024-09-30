use core::fmt;

pub enum CustomDbErr {
    NotFound,
}

impl fmt::Display for CustomDbErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomDbErr::NotFound => write!(f, "NotFound"),
        }
    }
}

impl std::str::FromStr for CustomDbErr {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NotFound" => Ok(CustomDbErr::NotFound),
            _ => Err("Unimplemented CustomDbErr"),
        }
    }
}
