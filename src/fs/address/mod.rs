use sea_orm::{DatabaseConnection, DbErr, EntityTrait, Select};

mod address_owner;
mod address_path;
mod resolved_address;

pub async fn find_one<E, T>(
    select: Select<E>,
    db: &DatabaseConnection,
    map: impl FnOnce(E::Model) -> T,
) -> Result<Option<T>, DbErr>
where
    E: EntityTrait,
{
    Ok(select.one(db).await?.map(map))
}
