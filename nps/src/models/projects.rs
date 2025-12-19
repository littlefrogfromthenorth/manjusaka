use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait,Set};
use serde::{Deserialize, Serialize};
use poem_openapi::{Object,OpenApi};
use std::convert::TryInto;



#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Default, Object)]
#[oai(rename = "ProjectModel")] 
#[sea_orm(table_name = "projects")]
pub struct Model {  
	#[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]  
	pub route 	 : String,
	pub name  	 : String,
	pub enckey 	 : String, 
	#[sea_orm(default_value = "")]
	pub listen   : String,
	#[sea_orm(default_value = "")]
	pub domain 	 : String, 
	#[sea_orm(default_value = "")]
	pub proxy 	 : String,  
	#[sea_orm(default_value = "")]
	pub callback1 : String, 
	#[sea_orm(default_value = "")]
	pub callback2 : String, 
	#[sea_orm(default_value = "")]
	pub config 	 : String,  
	#[sea_orm(default_value = true)]	
	pub isrun 	 : bool,  
	#[sea_orm(default_value = "")]
	pub note 	 : String, 
	  
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

impl Entity {
    pub fn find_by_route(route: &str) -> Select<Entity> {
        Self::find().filter(Column::Route.eq(route.to_string()))
    }
}

