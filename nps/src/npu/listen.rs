use super::{ApiResponse,JsonResponse};
use crate::unwrap_err;
use crate::models;
use crate::core::AppState;
use crate::npc::npc2;

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
    pub struct ListenQuery {
        pub page: Option<u64>,
        pub size: Option<u64>,
        pub keyword: Option<String>,
    }

    #[derive(Debug, Object, Deserialize, Serialize)]
    pub struct ListenQueryResult  {
        pub total : u64,
        pub items : Vec<models::listens::Model>,
    }

    #[derive(Debug, Object)]
    pub struct ListenAction {
        pub id: Option<String>,
        pub listenaddr: Option<String>,
        pub onlineaddr: Option<String>,
        pub proxyaddr: Option<String>,
        pub transport: Option<String>,
        pub enckey: Option<String>,
        pub noise: Option<String>,
        pub isrun: Option<bool>,
    }
}

pub struct ListenApi;

#[OpenApi]
impl ListenApi {
    #[oai(path = "/listenlist", method = "post")]
    pub async fn listenlist(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::ListenQuery>
    ) -> Result<JsonResponse<types::ListenQueryResult>> {
        let mut response = ApiResponse::default();

        let page = req.page.unwrap_or(1);
        let size = req.size.unwrap_or(20).min(100);

        let query = models::prelude::Listens::find();

        let count = query.clone();
        let total = unwrap_err!(count.count(&app.conn).await);

        let items = unwrap_err!(query
            .order_by_desc(models::listens::Column::Updateat)
            .offset((page - 1) * size)
            .limit(size)
            .all(&app.conn)
            .await);

        response.with_data(types::ListenQueryResult{
            total,
            items
        });

        Ok(Json(response))
    }

    #[oai(path = "/listendelete", method = "post")]
    pub async fn listendelete(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::ListenAction>
    ) -> Result<JsonResponse<String>> {
        let mut response = ApiResponse::default();
        let id = req.id.unwrap_or_default();

        app.stop_listen(&id).await?;
        let _ = unwrap_err!(models::prelude::Listens::delete_by_id(id).exec(&app.conn).await);
        
        response.with_msg("ok");
        Ok(Json(response))
    }


    #[oai(path = "/listenupdate", method = "post")]
    pub async fn listenupdate(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::ListenAction>
    ) -> Result<JsonResponse<models::listens::Model>> {
        let mut response = ApiResponse::default();

        let listen = unwrap_err!(models::prelude::Listens::find_by_id(req.id.unwrap_or_default()).one(&app.conn).await);

        let updated_listen = unwrap_err!(match listen {
            Some(listen) => {
                response.with_msg("update");
                let mut active_model: models::listens::ActiveModel = listen.into();
                if let Some(listenaddr) = req.listenaddr {
                    active_model.listenaddr = Set(listenaddr);
                }
                if let Some(onlineaddr) = req.onlineaddr {
                    active_model.onlineaddr = Set(onlineaddr);
                }
                if let Some(proxyaddr) = req.proxyaddr {
                    active_model.proxyaddr = Set(proxyaddr);
                }
                if let Some(transport) = req.transport {
                    active_model.transport = Set(transport);
                }
                if let Some(enckey) = req.enckey {
                    active_model.enckey = Set(enckey);
                }
                if let Some(noise) = req.noise {
                    active_model.noise = Set(noise);
                }
                if let Some(isrun) = req.isrun {
                    active_model.isrun = Set(isrun);
                }
                active_model.update(&app.conn).await
            },
            None => {
                response.with_msg("insert");
                let mut active_model = models::listens::ActiveModel{
                    id          : Set(utils::uuid()),
                    transport   : Set("tcp".to_string()),
                    listenaddr  : Set("0.0.0.0:32000".to_string()),
                    noise       : Set("Noise_KK_25519_ChaChaPoly_BLAKE2s".to_string()),
                    enckey      : Set(utils::randstr(8)),
                    .. Default::default()
                };
                if let Some(listenaddr) = req.listenaddr {
                    active_model.listenaddr = Set(listenaddr);
                }
                if let Some(onlineaddr) = req.onlineaddr {
                    active_model.onlineaddr = Set(onlineaddr);
                }
                if let Some(proxyaddr) = req.proxyaddr {
                    active_model.proxyaddr = Set(proxyaddr);
                }
                if let Some(transport) = req.transport {
                    active_model.transport = Set(transport);
                }
                if let Some(enckey) = req.enckey {
                    active_model.enckey = Set(enckey);
                }
                if let Some(noise) = req.noise {
                    active_model.noise = Set(noise);
                }
                active_model.insert(&app.conn).await
            }
        });

        response.with_data(updated_listen);
        Ok(Json(response))
    }

    
    #[oai(path = "/listenaction", method = "post")]
    pub async fn listenaction(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::ListenAction>
    ) -> Result<JsonResponse<String>> {
        let id = req.id.unwrap_or_default();
        let mut response = ApiResponse::default();
        if let Ok(Some(listen)) = models::prelude::Listens::find_by_id(&id).one(&app.conn).await {
            if req.isrun.unwrap_or_default(){
                unwrap_err!(npc2::start_listen(app.clone(), listen.clone()).await);
            }else{
                log::info!("remove listen {}",listen.listenaddr);
                unwrap_err!(app.stop_listen(&id).await);
            }
            let mut active_model: models::listens::ActiveModel = listen.into();
            if let Some(isrun) = req.isrun {
                active_model.isrun = Set(isrun);
            }
            unwrap_err!(active_model.update(&app.conn).await);
            response.with_msg("ok");
        }else{
            response.with_code(500).with_msg("id not find");
        }
        
        Ok(Json(response))
    }

}


