use super::{ApiResponse,JsonResponse};
use crate::unwrap_err;
use crate::models;
use crate::core::AppState;

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
use poem::session::Session;
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
    pub struct ProjectQuery {
        pub page: Option<u64>,
        pub size: Option<u64>,
        pub keyword: Option<String>,
    }

    #[derive(Debug, Object, Deserialize, Serialize)]
    pub struct ProjectQueryResult  {
        pub total : u64,
        pub items : Vec<models::projects::Model>,
    }

    #[derive(Debug, Object)]
    pub struct ProjectAction {
        pub id: Option<String>,
        pub route: Option<String>,
        pub name: Option<String>,
        pub enckey: Option<String>,
        pub listen: Option<String>,
        pub domain: Option<String>,
        pub proxy: Option<String>,
        pub callback1: Option<String>,
        pub callback2: Option<String>,
        pub config: Option<String>,
        pub isrun: Option<bool>,
        pub note: Option<String>,
    }
}

pub struct ProjectApi;

#[OpenApi]
impl ProjectApi {
    #[oai(path = "/projectlist", method = "post")]
    pub async fn projectlist(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::ProjectQuery>
    ) -> Result<JsonResponse<types::ProjectQueryResult>> {
        let mut response = ApiResponse::default();

        let page = req.page.unwrap_or(1);
        let size = req.size.unwrap_or(20).min(100);

        let mut query = models::prelude::Projects::find();

        if let Some(keyword) = req.keyword {
            let keyword = format!("%{}%", keyword);
            query = query.filter(
                Condition::any()
                    .add(models::projects::Column::Name.like(keyword.clone()))
                    .add(models::projects::Column::Note.like(keyword))
            );
        }

        let count = query.clone();
        let total = unwrap_err!(count.count(&app.conn).await);

        let items = unwrap_err!(query
            .order_by_desc(models::projects::Column::Updateat)
            .offset((page - 1) * size)
            .limit(size)
            .all(&app.conn)
            .await);

        response.with_data(types::ProjectQueryResult{
            total,
            items
        });

        Ok(Json(response))
    }

    #[oai(path = "/projectdelete", method = "post")]
    pub async fn projectdelete(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::ProjectAction>
    ) -> Result<JsonResponse<String>> {
        let mut response = ApiResponse::default();

        let _ = unwrap_err!(models::prelude::Projects::delete_by_id(req.id.unwrap_or_default())
                .exec(&app.conn)
                .await);

        response.with_msg("ok");
        Ok(Json(response))
    }

    #[oai(path = "/projectupdate", method = "post")]
    pub async fn projectupdate(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::ProjectAction>
    ) -> Result<JsonResponse<models::projects::Model>> {
        let mut response = ApiResponse::default();

        let project = unwrap_err!(models::prelude::Projects::find_by_id(req.id.unwrap_or_default())
            .one(&app.conn)
            .await);

        let updated_project = unwrap_err!(match project {
            Some(project) => {
                response.with_msg("update");
                let mut active_model: models::projects::ActiveModel = project.into();
                if let Some(route) = req.route {
                    active_model.route = Set(route);
                }
                if let Some(name) = req.name {
                    active_model.name = Set(name);
                }
                if let Some(listen) = req.listen {
                    active_model.listen = Set(listen);
                }
                if let Some(enckey) = req.enckey {
                    active_model.enckey = Set(enckey);
                }
                if let Some(domain) = req.domain {
                    active_model.domain = Set(domain);
                }
                if let Some(proxy) = req.proxy {
                    active_model.proxy = Set(proxy);
                }
                if let Some(callback1) = req.callback1 {
                    active_model.callback1 = Set(callback1);
                }
                if let Some(callback2) = req.callback2 {
                    active_model.callback2 = Set(callback2);
                }
                if let Some(config) = req.config {
                    active_model.config = Set(config);
                }
                if let Some(isrun) = req.isrun {
                    active_model.isrun = Set(isrun);
                }
                if let Some(note) = req.note {
                    active_model.note = Set(note);
                }
                active_model.update(&app.conn).await
            },
            None => {
                response.with_msg("insert");
                let mut active_model = models::projects::ActiveModel{
                    id       : Set(utils::uuid()),
                    route    : Set(utils::randstr(8)),
                    enckey   : Set(utils::randstr(16)),
                    .. Default::default()
                };
                if let Some(route) = req.route {
                    active_model.route = Set(route);
                }
                if let Some(enckey) = req.enckey {
                    active_model.enckey = Set(enckey);
                }
                active_model.name       = Set(req.name.unwrap_or_default());
                active_model.listen     = Set(req.listen.unwrap_or_default());
                active_model.callback1  = Set(req.callback1.unwrap_or_default());
                active_model.callback2  = Set(req.callback2.unwrap_or_default());
                active_model.domain     = Set(req.domain.unwrap_or_default());
                active_model.config     = Set(req.config.unwrap_or_default());
                active_model.proxy      = Set(req.proxy.unwrap_or_default());
                active_model.isrun      = Set(req.isrun.unwrap_or_default());
                active_model.note       = Set(req.note.unwrap_or_default());
                active_model.insert(&app.conn).await
            }
        });

        unwrap_err!(app.project_active(updated_project.clone()).await);

        response.with_data(updated_project);

        Ok(Json(response))
    }

    #[oai(path = "/projectself", method = "post")]
    pub async fn projectself(
        &self,
        sess: &Session,
        Json(req): Json<types::ProjectAction>
    ) -> JsonResponse<String> {
        let mut response = ApiResponse::default();
        sess.set("TARGET",req.id.unwrap_or_default());
        Json(response)
    }

}


