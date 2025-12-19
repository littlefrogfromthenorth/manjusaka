use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "plugins")]
#[oai(rename = "PluginModel")] 
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]  	 
	pub plugname : String,   
	pub plugtype : String,  
	pub plugargs : String,   
	pub plugos   : String,   
	pub plugarch : String,
	pub updateat : i64,   
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
