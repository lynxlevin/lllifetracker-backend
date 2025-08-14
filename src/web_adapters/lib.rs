mod journal;
mod middlewares;
mod my_way;
mod tags;
mod users;
mod utils;

pub use journal::{
    diaries::diary_routes, reading_notes::reading_note_routes, thinking_notes::thinking_note_routes,
};
pub use my_way::{
    action_goals::action_goal_routes, action_tracks::action_track_routes, actions::action_routes,
    ambitions::ambition_routes, desired_state_categories::desired_state_category_routes,
    desired_states::desired_state_routes,
};
pub use tags::tag_routes;
pub use users::auth_routes;

pub use middlewares::auth as auth_middleware;
