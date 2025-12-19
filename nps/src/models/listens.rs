use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait,Set};
use serde::{Deserialize, Serialize};
use poem_openapi::{Object,OpenApi};
use std::convert::TryInto;


#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Default, Object)]
#[oai(rename = "ListenModel")] 
#[sea_orm(table_name = "listens")]
pub struct Model {  
	#[sea_orm(primary_key, auto_increment = false)]
    pub id: String,

    #[sea_orm(unique)]  
	pub listenaddr 	 : String, //监听地址
	
	#[sea_orm(default_value = "")]
	pub onlineaddr 	 : String, //上线地址

	#[sea_orm(default_value = "")]
	pub proxyaddr 	 : String, //代理地址

	pub enckey 	 : String, 

	#[sea_orm(default_value = "tcp")]
	pub transport 	 : String,

	#[sea_orm(default_value = "Noise_KK_25519_ChaChaPoly_BLAKE2s")]
	pub noise 	 : String,	

	#[sea_orm(default_value = false)]	
	pub isrun 	 : bool, 

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



