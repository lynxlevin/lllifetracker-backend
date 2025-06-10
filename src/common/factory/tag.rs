use entities::tag;
use sea_orm::Set;
use uuid::Uuid;

pub fn tag(user_id: Uuid) -> tag::ActiveModel {
    tag::ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        name: Set(Some("plain_tag".to_string())),
        ..Default::default()
    }
}

pub trait TagFactory {
    fn name(self, name: Option<String>) -> tag::ActiveModel;
}

impl TagFactory for tag::ActiveModel {
    fn name(mut self, name: Option<String>) -> tag::ActiveModel {
        self.name = Set(name);
        self
    }
}
