mod actions;
mod ambitions;
mod health;
mod objectives;
mod users;
mod memos;
mod tags;

pub use actions::action_routes;
pub use ambitions::ambition_routes;
pub use health::health_check;
pub use objectives::objective_routes;
pub use users::auth_routes;
pub use memos::memo_routes;
pub use tags::tag_routes;
