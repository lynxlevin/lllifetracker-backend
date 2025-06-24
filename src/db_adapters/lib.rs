mod journal;
mod my_way;
pub mod tag_adapter;
pub mod user_adapter;

pub use journal::{diary_adapter, reading_note_adapter};
pub use my_way::{
    action_adapter, action_track_adapter, ambition_adapter, desired_state_adapter,
    desired_state_category_adapter,
};

use core::fmt;
pub use sea_orm::Order;

pub enum CustomDbErr {
    Duplicate,
    NotFound,
    Unimplemented,
}

impl fmt::Display for CustomDbErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomDbErr::Duplicate => write!(f, "Duplicate"),
            CustomDbErr::NotFound => write!(f, "NotFound"),
            CustomDbErr::Unimplemented => write!(f, "Unimplemented"),
        }
    }
}

impl std::str::FromStr for CustomDbErr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Duplicate" => Ok(CustomDbErr::Duplicate),
            "NotFound" => Ok(CustomDbErr::NotFound),
            _ => Ok(CustomDbErr::Unimplemented),
        }
    }
}

impl From<&String> for CustomDbErr {
    fn from(value: &String) -> Self {
        value.parse().unwrap()
    }
}
