use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(DeriveEntityModel, Debug, Clone)]
#[sea_orm(table_name = "topic")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub topic: String,
    #[sea_orm(has_many, via = "address_topic")] // M-N relation with junction
    pub addresses: HasMany<super::address::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
