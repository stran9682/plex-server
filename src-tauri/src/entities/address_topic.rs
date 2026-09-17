use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(DeriveEntityModel, Debug, Clone)]
#[sea_orm(table_name = "address_topic")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub address_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub topic_id: i32,
    #[sea_orm(belongs_to, from = "address_id", to = "id")]
    pub cake: BelongsTo<super::address::Entity>,
    #[sea_orm(belongs_to, from = "topic_id", to = "id")]
    pub filling: BelongsTo<super::topic::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
