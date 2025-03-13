use entities::{challenge, challenges_tags};
use chrono::Utc;
use sea_orm::{
    entity::prelude::*, DeriveColumn, EnumIter, QuerySelect, Set, TransactionError,
    TransactionTrait,
};

use super::challenge_query::ChallengeQuery;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    TagId,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewChallenge {
    pub title: String,
    pub text: String,
    pub date: chrono::NaiveDate,
    pub tag_ids: Vec<uuid::Uuid>,
    pub user_id: uuid::Uuid,
}

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct UpdateChallenge {
    pub id: uuid::Uuid,
    pub title: Option<String>,
    pub text: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub tag_ids: Option<Vec<uuid::Uuid>>,
    pub user_id: uuid::Uuid,
}

pub struct ChallengeMutation;

impl ChallengeMutation {
    pub async fn create(
        db: &DbConn,
        form_data: NewChallenge,
    ) -> Result<challenge::Model, TransactionError<DbErr>> {
        db.transaction::<_, challenge::Model, DbErr>(|txn| {
            Box::pin(async move {
                let now = Utc::now();
                let challenge_id = uuid::Uuid::new_v4();
                let created_challenge = challenge::ActiveModel {
                    id: Set(challenge_id),
                    user_id: Set(form_data.user_id),
                    title: Set(form_data.title.to_owned()),
                    text: Set(form_data.text.to_owned()),
                    date: Set(form_data.date),
                    archived: Set(false),
                    accomplished_at: Set(None),
                    created_at: Set(now.into()),
                    updated_at: Set(now.into()),
                }
                .insert(txn)
                .await?;

                for tag_id in form_data.tag_ids {
                    challenges_tags::ActiveModel {
                        challenge_id: Set(created_challenge.id),
                        tag_id: Set(tag_id),
                    }
                    .insert(txn)
                    .await?;
                }

                Ok(created_challenge)
            })
        })
        .await
    }

    pub async fn partial_update(
        db: &DbConn,
        form: UpdateChallenge,
    ) -> Result<challenge::Model, TransactionError<DbErr>> {
        let challenge_result =
            ChallengeQuery::find_by_id_and_user_id(db, form.id, form.user_id).await;
        db.transaction::<_, challenge::Model, DbErr>(|txn| {
            Box::pin(async move {
                let mut challenge: challenge::ActiveModel = challenge_result?.into();
                if let Some(title) = form.title {
                    challenge.title = Set(title);
                }
                if let Some(text) = form.text {
                    challenge.text = Set(text);
                }
                if let Some(date) = form.date {
                    challenge.date = Set(date);
                }
                if let Some(tag_ids) = form.tag_ids {
                    let linked_tag_ids = challenges_tags::Entity::find()
                        .column_as(challenges_tags::Column::TagId, QueryAs::TagId)
                        .filter(challenges_tags::Column::ChallengeId.eq(form.id))
                        .into_values::<uuid::Uuid, QueryAs>()
                        .all(txn)
                        .await?;

                    let tag_links_to_create: Vec<challenges_tags::ActiveModel> = tag_ids
                        .clone()
                        .into_iter()
                        .filter(|id| !linked_tag_ids.contains(id))
                        .map(|tag_id| challenges_tags::ActiveModel {
                            challenge_id: Set(form.id),
                            tag_id: Set(tag_id),
                        })
                        .collect();
                    challenges_tags::Entity::insert_many(tag_links_to_create)
                        .on_empty_do_nothing()
                        .exec(txn)
                        .await?;

                    let ids_to_delete: Vec<uuid::Uuid> = linked_tag_ids
                        .into_iter()
                        .filter(|linked_tag_id| !tag_ids.contains(linked_tag_id))
                        .collect();
                    if ids_to_delete.len() > 0 {
                        challenges_tags::Entity::delete_many()
                            .filter(challenges_tags::Column::ChallengeId.eq(form.id))
                            .filter(challenges_tags::Column::TagId.is_in(ids_to_delete))
                            .exec(txn)
                            .await?;
                    }
                }
                challenge.updated_at = Set(Utc::now().into());
                challenge.update(txn).await
            })
        })
        .await
    }

    pub async fn delete(
        db: &DbConn,
        challenge_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        ChallengeQuery::find_by_id_and_user_id(db, challenge_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }

    pub async fn archive(
        db: &DbConn,
        challenge_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<challenge::Model, DbErr> {
        let mut challenge: challenge::ActiveModel =
            ChallengeQuery::find_by_id_and_user_id(db, challenge_id, user_id)
                .await?
                .into();
        challenge.archived = Set(true);
        challenge.updated_at = Set(Utc::now().into());
        challenge.update(db).await
    }

    pub async fn mark_accomplished(
        db: &DbConn,
        challenge_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<challenge::Model, DbErr> {
        let mut challenge: challenge::ActiveModel =
            ChallengeQuery::find_by_id_and_user_id(db, challenge_id, user_id)
                .await?
                .into();
        challenge.accomplished_at = Set(Some(Utc::now().into()));
        challenge.updated_at = Set(Utc::now().into());
        challenge.update(db).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use sea_orm::DbErr;

    use test_utils::{self, *};
    use ::types::CustomDbErr;

    use super::*;

    /// Asserts equality of the following fields.
    /// ```
    /// id
    /// title
    /// text
    /// date
    /// archived
    /// accomplished_at.is_some()
    /// accomplished_at: actual > expected if both are Some.
    /// user_id
    /// created_at
    /// updated_at: actual > expected
    /// ```
    fn assert_updated(actual: &challenge::Model, expected: &challenge::Model) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.title, expected.title);
        assert_eq!(actual.text, expected.text);
        assert_eq!(actual.date, expected.date);
        assert_eq!(actual.archived, expected.archived);
        assert_eq!(
            actual.accomplished_at.is_some(),
            expected.accomplished_at.is_some()
        );
        if actual.accomplished_at.is_some() {
            assert!(actual.accomplished_at.unwrap() > expected.accomplished_at.unwrap());
        }
        assert_eq!(actual.user_id, expected.user_id);
        assert_eq!(actual.created_at, expected.created_at);
        assert!(actual.updated_at > expected.updated_at);
    }

    #[actix_web::test]
    async fn create() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let (_, tag_0) = factory::action(user.id)
            .name("action_0".to_string())
            .insert_with_tag(&db)
            .await?;
        let (_, tag_1) = factory::action(user.id)
            .name("action_1".to_string())
            .insert_with_tag(&db)
            .await?;

        let challenge_title = "New Mission Memo".to_string();
        let challenge_text = "This is a new mission memo for testing create method.".to_string();
        let today = chrono::Utc::now().date_naive();

        let form_data = NewChallenge {
            title: challenge_title.clone(),
            text: challenge_text.clone(),
            date: today,
            tag_ids: vec![tag_0.id, tag_1.id],
            user_id: user.id,
        };

        let returned_challenge = ChallengeMutation::create(&db, form_data).await.unwrap();
        assert_eq!(returned_challenge.title, challenge_title.clone());
        assert_eq!(returned_challenge.text, challenge_text.clone());
        assert_eq!(returned_challenge.date, today);
        assert_eq!(returned_challenge.archived, false);
        assert_eq!(returned_challenge.accomplished_at, None);
        assert_eq!(returned_challenge.user_id, user.id);

        let created_challenge = challenge::Entity::find_by_id(returned_challenge.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_challenge.title, challenge_title.clone());
        assert_eq!(created_challenge.text, challenge_text.clone());
        assert_eq!(created_challenge.date, today);
        assert_eq!(created_challenge.archived, false);
        assert_eq!(created_challenge.accomplished_at, None);
        assert_eq!(created_challenge.user_id, user.id);
        assert_eq!(
            created_challenge.created_at,
            returned_challenge.created_at
        );
        assert_eq!(
            created_challenge.updated_at,
            returned_challenge.updated_at
        );

        let linked_tag_ids: Vec<uuid::Uuid> = challenges_tags::Entity::find()
            .column_as(challenges_tags::Column::TagId, QueryAs::TagId)
            .filter(challenges_tags::Column::ChallengeId.eq(returned_challenge.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 2);
        assert!(linked_tag_ids.contains(&tag_0.id));
        assert!(linked_tag_ids.contains(&tag_1.id));

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_title() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;

        let form = UpdateChallenge {
            id: challenge.id,
            title: Some("Updated Mission Memo".to_string()),
            text: None,
            date: None,
            tag_ids: None,
            user_id: user.id,
        };
        let mut expected = challenge.clone();
        expected.title = form.title.clone().unwrap();

        let returned_challenge = ChallengeMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_challenge, &expected);

        let updated_challenge = challenge::Entity::find_by_id(challenge.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_challenge, &expected);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_text() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;

        let form = UpdateChallenge {
            id: challenge.id,
            title: None,
            text: Some("Updated mission memo content.".to_string()),
            date: None,
            tag_ids: None,
            user_id: user.id,
        };

        let mut expected = challenge.clone();
        expected.text = form.text.clone().unwrap();

        let returned_challenge = ChallengeMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_challenge, &expected);

        let updated_challenge = challenge::Entity::find_by_id(challenge.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_challenge, &expected);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_date() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;

        let form = UpdateChallenge {
            id: challenge.id,
            title: None,
            text: None,
            date: Some(chrono::Utc::now().with_year(1900).unwrap().date_naive()),
            tag_ids: None,
            user_id: user.id,
        };

        let mut expected = challenge.clone();
        expected.date = form.date.unwrap();

        let returned_challenge = ChallengeMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_challenge, &expected);

        let updated_challenge = challenge::Entity::find_by_id(challenge.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_challenge, &expected);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_add_tags() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;

        let form = UpdateChallenge {
            id: challenge.id,
            title: None,
            text: None,
            date: None,
            tag_ids: Some(vec![ambition_tag.id]),
            user_id: user.id,
        };

        let expected = challenge.clone();

        let returned_challenge = ChallengeMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_challenge, &expected);

        let updated_challenge = challenge::Entity::find_by_id(challenge.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_challenge, &expected);

        let linked_tag_ids: Vec<uuid::Uuid> = challenges_tags::Entity::find()
            .column_as(challenges_tags::Column::TagId, QueryAs::TagId)
            .filter(challenges_tags::Column::ChallengeId.eq(returned_challenge.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 1);
        assert!(linked_tag_ids.contains(&ambition_tag.id));

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_remove_tags() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        factory::link_challenge_tag(&db, challenge.id, ambition_tag.id).await?;

        let form = UpdateChallenge {
            id: challenge.id,
            title: None,
            text: None,
            date: None,
            tag_ids: Some(vec![]),
            user_id: user.id,
        };

        let expected = challenge.clone();

        let returned_challenge = ChallengeMutation::partial_update(&db, form.clone())
            .await
            .unwrap();
        assert_updated(&returned_challenge, &expected);

        let updated_challenge = challenge::Entity::find_by_id(challenge.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_challenge, &expected);

        let linked_tag_ids: Vec<uuid::Uuid> = challenges_tags::Entity::find()
            .column_as(challenges_tags::Column::TagId, QueryAs::TagId)
            .filter(challenges_tags::Column::ChallengeId.eq(returned_challenge.id))
            .into_values::<_, QueryAs>()
            .all(&db)
            .await?;
        assert_eq!(linked_tag_ids.len(), 0);

        Ok(())
    }

    #[actix_web::test]
    async fn partial_update_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;
        let form = UpdateChallenge {
            id: challenge.id,
            title: None,
            text: None,
            date: None,
            tag_ids: None,
            user_id: uuid::Uuid::new_v4(),
        };

        let error = ChallengeMutation::partial_update(&db, form)
            .await
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            TransactionError::Transaction(DbErr::Custom(CustomDbErr::NotFound.to_string()))
                .to_string(),
        );

        Ok(())
    }

    #[actix_web::test]
    async fn delete() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        factory::link_challenge_tag(&db, challenge.id, ambition_tag.id).await?;

        ChallengeMutation::delete(&db, challenge.id, user.id).await?;

        let challenge_in_db = challenge::Entity::find_by_id(challenge.id)
            .one(&db)
            .await?;
        assert!(challenge_in_db.is_none());

        let challenges_tags_in_db = challenges_tags::Entity::find()
            .filter(challenges_tags::Column::ChallengeId.eq(challenge.id))
            .filter(challenges_tags::Column::TagId.eq(ambition_tag.id))
            .one(&db)
            .await?;
        assert!(challenges_tags_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;

        let error = ChallengeMutation::delete(&db, challenge.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn archive() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;

        let mut expected = challenge.clone();
        expected.archived = true;

        let returned_challenge = ChallengeMutation::archive(&db, challenge.id, user.id)
            .await
            .unwrap();
        assert_updated(&returned_challenge, &expected);

        let updated_challenge = challenge::Entity::find_by_id(challenge.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_challenge, &expected);

        Ok(())
    }

    #[actix_web::test]
    async fn archive_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;

        let error = ChallengeMutation::archive(&db, challenge.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn mark_accomplished() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;

        let mut expected = challenge.clone();
        expected.accomplished_at = Some(Utc::now().into());

        let returned_challenge =
            ChallengeMutation::mark_accomplished(&db, challenge.id, user.id)
                .await
                .unwrap();
        assert_updated(&returned_challenge, &expected);

        let updated_challenge = challenge::Entity::find_by_id(challenge.id)
            .one(&db)
            .await?
            .unwrap();
        assert_updated(&updated_challenge, &expected);

        Ok(())
    }

    #[actix_web::test]
    async fn mark_accomplished_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge = factory::challenge(user.id).insert(&db).await?;

        let error =
            ChallengeMutation::mark_accomplished(&db, challenge.id, uuid::Uuid::new_v4())
                .await
                .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }
}
