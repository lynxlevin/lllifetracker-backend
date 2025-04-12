mod action_tracks;
mod actions;
mod ambitions;
mod desired_states;
mod diaries;
mod health;
mod reading_notes;
mod tags;
mod users;

pub use action_tracks::action_track_routes;
pub use actions::action_routes;
pub use ambitions::ambition_routes;
pub use desired_states::desired_state_routes;
pub use diaries::diary_routes;
pub use health::health_check;
pub use reading_notes::reading_note_routes;
pub use tags::tag_routes;
pub use users::auth_routes;
