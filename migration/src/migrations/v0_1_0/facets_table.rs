use crate::{migrations::UUID_DEFAULT, util::TableMigration};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum Facet {
    Table,
    Key,
}

#[derive(Iden)]
enum FacetAddressType {
    Enum,
    InNote,
    Shard,
}

pub struct FacetTableMigration;

#[async_trait::async_trait]
impl TableMigration for FacetTableMigration {
    fn get_name(&self) -> String {
        Facet::Table.unquoted().to_string()
    }

    fn def_table<'a>(&self, table: &'a mut TableCreateStatement) -> &'a mut TableCreateStatement {
        table
            .col(pk_auto("key").unique_key())
            .col(uuid_uniq("id").default(Expr::cust(UUID_DEFAULT)))
            .col(text("header_field_name"))
            .col(enumeration(
                "facet_address_type",
                FacetAddressType::Enum,
                [FacetAddressType::InNote, FacetAddressType::Shard],
            ))
    }
}
