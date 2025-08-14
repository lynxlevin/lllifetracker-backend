use chrono::Utc;
use entities::thinking_note;
use sea_orm::Set;
use uuid::Uuid;

pub fn thinking_note(user_id: Uuid) -> thinking_note::ActiveModel {
    let now = Utc::now();
    thinking_note::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        question: Set(Some("question".to_string())),
        thought: Set(None),
        answer: Set(None),
        user_id: Set(user_id),
        resolved_at: Set(None),
        archived_at: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait ThinkingNoteFactory {
    fn question(self, question: Option<String>) -> thinking_note::ActiveModel;
}

impl ThinkingNoteFactory for thinking_note::ActiveModel {
    fn question(mut self, question: Option<String>) -> thinking_note::ActiveModel {
        self.question = Set(question);
        self
    }
}
