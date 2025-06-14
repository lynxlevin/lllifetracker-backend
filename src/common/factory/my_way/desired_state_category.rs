use entities::desired_state_category;
use sea_orm::Set;
use uuid::Uuid;

pub fn desired_state_category(user_id: Uuid) -> desired_state_category::ActiveModel {
    let id = Uuid::now_v7();
    desired_state_category::ActiveModel {
        id: Set(id),
        user_id: Set(user_id),
        name: Set(format!("category-{}", id)),
        ordering: Set(None),
    }
}

pub trait DesiredStateCategoryFactory {
    fn name(self, name: String) -> desired_state_category::ActiveModel;
    fn ordering(self, ordering: Option<i32>) -> desired_state_category::ActiveModel;
}

impl DesiredStateCategoryFactory for desired_state_category::ActiveModel {
    fn name(mut self, name: String) -> desired_state_category::ActiveModel {
        self.name = Set(name);
        self
    }

    fn ordering(mut self, ordering: Option<i32>) -> desired_state_category::ActiveModel {
        self.ordering = Set(ordering);
        self
    }
}
