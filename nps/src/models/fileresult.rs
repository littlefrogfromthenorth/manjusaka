use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::Set;
use poem_openapi::{Object,OpenApi};
use std::convert::TryInto;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Default, Object)]
#[sea_orm(table_name = "fileresult")]
#[oai(rename = "FileResultModel")] 
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
    pub id: String, 	
	#[sea_orm(default_value = "")]  
	pub target   : String, 
	#[sea_orm(default_value = "")]  
	pub agent  	 : String,
	#[sea_orm(default_value = "")] 
	pub fileauth : String, 
	   
	pub filename : String, 
	#[sea_orm(default_value = "")]   
	pub filepath : String,
	#[sea_orm(default_value = 0)] 
	pub filesize : i64,
	  
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


