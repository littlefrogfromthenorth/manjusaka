use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::Set;
use poem_openapi::{Object,OpenApi};
use std::convert::TryInto;


#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Default, Object)]
#[sea_orm(table_name = "passresult")]
#[oai(rename = "PassResultModel")] 
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id 		 : String,
	#[sea_orm(default_value = "")]   
	pub target 	 : String,
	#[sea_orm(default_value = "")]   
	pub agent  	 : String,
	#[sea_orm(default_value = "")] 
	pub username : String, 
	  
	pub password : String,
	#[sea_orm(default_value = "")]   
	pub passtype : String, 
	#[sea_orm(default_value = "")]  
	pub passfrom : String,  
	pub updateat : i64, 
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, _insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        self.updateat = Set(utils::timestamp());
        Ok(self)
    }
}
