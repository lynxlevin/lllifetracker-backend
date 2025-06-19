mod my_way;

pub use my_way::action_track_adapter;

use core::fmt;
pub use sea_orm::Order;

pub enum CustomDbErr {
    Duplicate,
    Unimplemented,
}

impl fmt::Display for CustomDbErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomDbErr::Duplicate => write!(f, "Duplicate"),
            CustomDbErr::Unimplemented => write!(f, "Unimplemented"),
        }
    }
}

impl std::str::FromStr for CustomDbErr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Duplicate" => Ok(CustomDbErr::Duplicate),
            _ => Ok(CustomDbErr::Unimplemented),
        }
    }
}

impl From<&String> for CustomDbErr {
    fn from(value: &String) -> Self {
        value.parse().unwrap()
    }
}
