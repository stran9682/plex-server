use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(DeriveEntityModel, Debug, Clone)]
#[sea_orm(table_name = "address")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub endpoint: String,
    #[sea_orm(has_many, via = "address_topic")]
    pub topics: HasMany<super::topic::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
