use super::ActionVisible;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ObjectiveVisible {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ObjectiveVisibleWithActions {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub actions: Vec<ActionVisible>,
}

impl ObjectiveVisibleWithActions {
    pub fn push_action(&mut self, action: ActionVisible) {
        self.actions.push(action);
    }
}
