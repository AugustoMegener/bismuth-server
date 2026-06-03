use sea_orm::{entity::prelude::*, Set};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, sea_orm::EnumIter, sea_orm::DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)", rename_all = "camelCase")]
enum FacetAddressType {
    InNote, Shard
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "facets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub key: i64,
    pub id: Uuid,
    pub header_field_name: String,
    pub address_type: FacetAddressType
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {

    async fn before_save<C: ConnectionTrait>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr> {
        if insert && self.id.is_not_set() {
            self.id = Set(Uuid::new_v4());
        }
        Ok(self)
    }
}
