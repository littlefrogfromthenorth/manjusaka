use crate::unwrap_err;
use super::{ApiResponse,JsonResponse};
use crate::models;
use crate::core::AppState;


use poem::{
    web::Data,
    http::StatusCode,
    Error,
    Result
};
use poem::session::Session;
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
    pub struct PassQuery {
        pub page: Option<u64>,
        pub size: Option<u64>,
        pub keyword: Option<String>,
    }

    #[derive(Debug, Object, Deserialize, Serialize)]
    pub struct PassQueryResult  {
        pub total : u64,
        pub items : Vec<models::passresult::Model>,
    }

    #[derive(Debug, Object, Deserialize, Serialize)]
    pub struct PassAction {
        pub id: Option<String>,    
        pub username: Option<String>,
        pub password: Option<String>,
        pub passtype: Option<String>,
        pub passfrom: Option<String>,
    }
}

pub struct PassApi;

#[OpenApi]
impl PassApi {
    #[oai(path = "/passlist", method = "post")]
    pub async fn passlist(
        &self,
        sess: &Session,
        app: Data<&AppState>,
        Json(req): Json<types::PassQuery>
    ) -> Result<JsonResponse<types::PassQueryResult>> {
        let mut response = ApiResponse::default();

        let page = req.page.unwrap_or(1);
        let size = req.size.unwrap_or(20).min(100);

        let mut query = models::prelude::PassResult::find();

        query = query.filter(
            models::passresult::Column::Target.eq(
                sess.get::<String>("TARGET").unwrap_or_default())); //限定只能查看当前项目密码

        if let Some(keyword) = req.keyword {
            let keyword = format!("%{}%", keyword);
            query = query.filter(
                Condition::any()
                    .add(models::passresult::Column::Username.like(keyword.clone()))
                    .add(models::passresult::Column::Password.like(keyword.clone()))
                    .add(models::passresult::Column::Passtype.like(keyword.clone()))
                    .add(models::passresult::Column::Passfrom.like(keyword))
            );
        }

        let count = query.clone();
        let total = unwrap_err!(count.count(&app.conn).await);

        let items = unwrap_err!(query
            .order_by_desc(models::passresult::Column::Updateat)
            .offset((page - 1) * size)
            .limit(size)
            .all(&app.conn)
            .await);

        response.with_data(types::PassQueryResult{
            total,
            items
        });

        Ok(Json(response))
    }

    #[oai(path = "/passdelete", method = "post")]
    pub async fn passdelete(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::PassAction>
    ) -> Result<JsonResponse<String>> {
        let mut response = ApiResponse::default();

        let _ = unwrap_err!(models::prelude::PassResult::delete_by_id(req.id.unwrap_or_default()).exec(&app.conn).await);

        response.with_msg("ok");
        Ok(Json(response))
    }

    #[oai(path = "/passupdate", method = "post")]
    pub async fn passupdate(
        &self,
        sess: &Session,
        app: Data<&AppState>,
        Json(req): Json<types::PassAction>
    ) -> Result<JsonResponse<models::passresult::Model>> {
        let mut response = ApiResponse::default();

        let passresult = unwrap_err!(models::prelude::PassResult::find_by_id(req.id.unwrap_or_default()).one(&app.conn).await);

        let updated_passresult = unwrap_err!(match passresult {
            Some(passresult) => {
                response.with_msg("update");
                let mut active_model: models::passresult::ActiveModel = passresult.into();
  
                if let Some(username) = req.username {
                    active_model.username = Set(username);
                }
                if let Some(password) = req.password {
                    active_model.password = Set(password);
                }
                if let Some(passtype) = req.passtype {
                    active_model.passtype = Set(passtype);
                }
                if let Some(passfrom) = req.passfrom {
                    active_model.passfrom = Set(passfrom);
                }
                active_model.update(&app.conn).await
            },
            None => {
                response.with_msg("insert");
                let mut active_model = models::passresult::ActiveModel{
                    id       : Set(utils::uuid()),
                    .. Default::default()
                };
                active_model.target    = Set(sess.get::<String>("TARGET").unwrap_or_default());
                active_model.agent     = Set("NPU".to_string());
                active_model.username  = Set(req.username.unwrap_or_default());
                active_model.password  = Set(req.password.unwrap_or_default());
                active_model.passtype  = Set(req.passtype.unwrap_or_default());
                active_model.passfrom  = Set(req.passfrom.unwrap_or_default());
                active_model.insert(&app.conn).await
            }
        });

        response.with_data(updated_passresult);

        Ok(Json(response))
    }

}


