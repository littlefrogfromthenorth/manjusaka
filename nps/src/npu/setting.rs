use super::{ApiResponse,JsonResponse};
use crate::models;
use crate::core::AppState;
use crate::unwrap_err;

use poem::{
    web::Data,
    http::StatusCode,
    Error,
    Result
};

use poem_openapi::{
    OpenApi, 
    payload::Json,
};
use sea_orm::{
    Set,
    DatabaseConnection,
    EntityTrait,
    ColumnTrait,
    PaginatorTrait,
    ActiveModelTrait,
    QueryFilter,
    QueryOrder,
    QuerySelect,
    Condition
};


pub mod types {
    use poem_openapi::Object;
    use serde::{Deserialize, Serialize};
    use crate::models;

    #[derive(Debug, Object)]
    pub struct SettingQuery {
        pub page: Option<u64>,
        pub size: Option<u64>,
        pub keyword: Option<String>,
    }

    #[derive(Debug, Object, Deserialize, Serialize)]
    pub struct QueryResult  {
        pub total : u64,
        pub items : Vec<models::settings::Model>,
    }

    #[derive(Debug, Object)]
    pub struct SettingData {
        pub key: String,
        pub value: String,
    }

    #[derive(Debug, Object)]
    pub struct SettingAction {
        pub id: String,
        pub data: Option<Vec<SettingData>>,
    }
}

pub struct SettingApi;

#[OpenApi]
impl SettingApi {
    #[oai(path = "/settinglist", method = "post")]
    pub async fn settinglist(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::SettingQuery>
    ) -> Result<JsonResponse<types::QueryResult>> {
        let mut response = ApiResponse::default();

        let page = req.page.unwrap_or(1);
        let size = req.size.unwrap_or(100);

        let mut query = models::prelude::Settings::find();

        if let Some(keyword) = req.keyword {
            let keyword = format!("%{}%", keyword);
            query = query.filter(
                Condition::any()
                    .add(models::settings::Column::Key.like(keyword.clone()))
                    .add(models::settings::Column::Keynote.like(keyword.clone()))
                    .add(models::settings::Column::Valuenote.like(keyword))
            );
        }

        let count = query.clone();
        let total = unwrap_err!(count.count(&app.conn).await);

        let items = unwrap_err!(query
            .offset((page - 1) * size)
            .limit(size)
            .all(&app.conn)
            .await);

        response.with_data(types::QueryResult{
            total,
            items
        });

        Ok(Json(response))
    }

    #[oai(path = "/settingdelete", method = "post")]
    pub async fn settingdelete(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::SettingAction>
    ) -> Result<JsonResponse<String>> {
        let mut response = ApiResponse::default();

        let result = unwrap_err!(models::prelude::Settings::delete_by_id(req.id)
            .exec(&app.conn)
            .await);

        response.with_msg("ok");
        Ok(Json(response))
    }

    #[oai(path = "/settingupdate", method = "post")]
    pub async fn settingupdate(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::SettingData>
    ) -> Result<JsonResponse<models::settings::Model>> {
        let mut response = ApiResponse::default();

        let setting = unwrap_err!(models::settings::set(&app.conn,&req.key,&req.value,None,None,None)
            .await); 
        response.with_data(setting);

        Ok(Json(response))
    }

}


