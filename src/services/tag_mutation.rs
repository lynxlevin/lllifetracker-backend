use entities::tag;
use sea_orm::{ActiveModelTrait, DbConn, DbErr, Set};
use types::{TagType, TagVisible};
use uuid::Uuid;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewTag {
    pub name: String,
    pub user_id: Uuid,
}

pub struct TagMutation;

impl TagMutation {
    pub async fn create(db: &DbConn, form_data: NewTag) -> Result<TagVisible, DbErr> {
        let created_tag = tag::ActiveModel {
            id: Set(Uuid::now_v7()),
            user_id: Set(form_data.user_id),
            name: Set(Some(form_data.name.to_owned())),
            ..Default::default()
        }
        .insert(db)
        .await?;
        Ok(TagVisible {
            id: created_tag.id,
            name: created_tag.name.unwrap(),
            tag_type: TagType::Plain,
            created_at: created_tag.created_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{DbErr, EntityTrait};

    use entities::tag;
    use test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn create() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;

        let form_data = NewTag {
            name: "new tag".to_string(),
            user_id: user.id,
        };

        let res = TagMutation::create(&db, form_data.clone()).await.unwrap();
        assert_eq!(res.name, form_data.name.clone());
        assert_eq!(res.tag_type, TagType::Plain);

        let tag_in_db = tag::Entity::find_by_id(res.id).one(&db).await?.unwrap();
        assert_eq!(tag_in_db.name, Some(form_data.name));
        assert_eq!(tag_in_db.user_id, user.id);
        Ok(())
    }
}
