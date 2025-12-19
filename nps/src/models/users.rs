use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait,Set,DbErr};
use serde::{Deserialize, Serialize};
use poem_openapi::Object;
use std::convert::TryInto;



#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Default, Object)]
#[sea_orm(table_name = "users")]
#[oai(rename = "UserModel")] 
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(unique)]
    pub username : String,
    pub password : String,
    #[sea_orm(default_value = "")]  
    pub userauth : String, 
    #[sea_orm(default_value = "")]   
    pub userrole : String,  
    #[sea_orm(default_value = "")]  
    pub userpid  : String, 
    #[sea_orm(default_value = "")]  
    pub email    : String,
    #[sea_orm(default_value = "")]  
    pub phone    : String,
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

impl Model{
    pub fn verify_password(
        &self, 
        username: &str, 
        password_hash: &str
    ) -> bool {
        self.password == utils::sha256(format!("\r{}\t{}\n",username, password_hash).as_bytes())
    }

    pub async fn update_password(
        self,
        db: &impl ConnectionTrait,
        username: &str, 
        password_hash: &str
    ) -> Result<Self, DbErr>{
        let mut active_model: ActiveModel = self.into();
        active_model.password = Set(utils::sha256(format!("\r{}\t{}\n", username, password_hash).as_bytes()));
        active_model.update(db).await
    }
}

