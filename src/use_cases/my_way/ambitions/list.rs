use crate::{
    my_way::ambitions::types::{AmbitionListQuery, AmbitionVisible},
    UseCaseError,
};
use db_adapters::{
    ambition_adapter::{AmbitionAdapter, AmbitionFilter, AmbitionOrder, AmbitionQuery},
    Order::Asc,
};
use entities::user as user_entity;

pub async fn list_ambitions<'a>(
    user: user_entity::Model,
    params: AmbitionListQuery,
    ambition_adapter: AmbitionAdapter<'a>,
) -> Result<Vec<AmbitionVisible>, UseCaseError> {
    match ambition_adapter
        .filter_eq_user(&user)
        .filter_eq_archived(params.show_archived_only.unwrap_or(false))
        .order_by_ordering_nulls_last(Asc)
        .order_by_created_at(Asc)
        .get_all()
        .await
    {
        Ok(ambitions) => Ok(ambitions
            .iter()
            .map(|ambition| AmbitionVisible::from(ambition))
            .collect::<Vec<_>>()),
        Err(e) => Err(UseCaseError::InternalServerError(format!("{:?}", e))),
    }
}
