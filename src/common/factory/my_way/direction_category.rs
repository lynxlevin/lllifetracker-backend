use entities::direction_category;
use sea_orm::Set;
use uuid::Uuid;

pub fn direction_category(user_id: Uuid) -> direction_category::ActiveModel {
    let id = Uuid::now_v7();
    direction_category::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        name: Set(format!("category-{}", id)),
        ordering: Set(None),
    }
}

pub trait DesiredStateCategoryFactory {
    fn name(self, name: String) -> direction_category::ActiveModel;
    fn ordering(self, ordering: Option<i32>) -> direction_category::ActiveModel;
}

impl DesiredStateCategoryFactory for direction_category::ActiveModel {
    fn name(mut self, name: String) -> direction_category::ActiveModel {
        self.name = Set(name);
        self
    }

    fn ordering(mut self, ordering: Option<i32>) -> direction_category::ActiveModel {
        self.ordering = Set(ordering);
        self
    }
}
