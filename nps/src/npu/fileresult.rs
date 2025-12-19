use super::{ApiResponse,JsonResponse};
use crate::unwrap_err;
use crate::models;
use crate::core::AppState;

use poem::{
    web::{Data, Query, Path, Multipart},
    http::StatusCode,
    Error,
    Result,
    
};

use poem_openapi::{
    OpenApi, 
    Multipart as ApiMultipart,
    payload::{Json,Attachment,AttachmentType},
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
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub mod types {
    use poem_openapi::Object;
    use serde::{Deserialize, Serialize};
    use crate::models;

    #[derive(Debug, Object)]
    pub struct FileQuery {
        pub page: Option<u64>,
        pub size: Option<u64>,
        pub keyword: Option<String>,
    }

    #[derive(Debug, Object, Deserialize, Serialize)]
    pub struct FileQueryResult  {
        pub total : u64,
        pub items : Vec<models::fileresult::Model>,
    }

    #[derive(Debug, Object, Deserialize, Serialize)]
    pub struct FileAction {
        pub id: String,
    }
}

pub struct FileApi;

#[OpenApi]
impl FileApi {
    #[oai(path = "/ufilelist", method = "post")]
    pub async fn ufilelist(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::FileQuery>
    ) -> Result<JsonResponse<types::FileQueryResult>> {
        let mut response = ApiResponse::default();

        let page = req.page.unwrap_or(1);
        let size = req.size.unwrap_or(20).min(100);

        let mut query = models::prelude::FileResult::find();
        query = query.filter(models::fileresult::Column::Agent.eq("NPU_UPLOAD"));

        if let Some(keyword) = req.keyword {
            let keyword = format!("%{}%", keyword);
            query = query.filter(models::fileresult::Column::Filename.like(keyword.clone()));
        }

        let count = query.clone();
        let total = unwrap_err!(count.count(&app.conn).await);

        let items = unwrap_err!(query
            .order_by_desc(models::fileresult::Column::Updateat)
            .offset((page - 1) * size)
            .limit(size)
            .all(&app.conn)
            .await);

        response.with_data(types::FileQueryResult{
            total,
            items
        });

        Ok(Json(response))
    }

    #[oai(path = "/filelist", method = "post")]
    pub async fn filelist(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::FileQuery>
    ) -> Result<JsonResponse<types::FileQueryResult>> {
        let mut response = ApiResponse::default();

        let page = req.page.unwrap_or(1);
        let size = req.size.unwrap_or(20).min(100);

        let mut query = models::prelude::FileResult::find();
        query = query.filter(models::fileresult::Column::Agent.ne("NPU_UPLOAD"));

        if let Some(keyword) = req.keyword {
            let keyword = format!("%{}%", keyword);
            query = query.filter(
                Condition::any()
                    .add(models::fileresult::Column::Filename.like(keyword.clone()))
                    .add(models::fileresult::Column::Filepath.like(keyword))
            );
        }

        let count = query.clone();
        let total = unwrap_err!(count.count(&app.conn).await);

        let items = unwrap_err!(query
            .order_by_desc(models::fileresult::Column::Updateat)
            .offset((page - 1) * size)
            .limit(size)
            .all(&app.conn)
            .await);

        response.with_data(types::FileQueryResult{
            total,
            items
        });

        Ok(Json(response))
    }

    #[oai(path = "/filedelete", method = "post")]
    pub async fn filedelete(
        &self,
        app: Data<&AppState>,
        Json(req): Json<types::FileAction>
    ) -> Result<JsonResponse<String>> {
        let mut response = ApiResponse::default();

        let ret = unwrap_err!(models::prelude::FileResult::delete_by_id(&req.id).exec(&app.conn).await);

        if ret.rows_affected == 1 {
            let path = std::path::PathBuf::from("./data").join(req.id); //有没有问题？
            let _ = std::fs::remove_file(path);
        }

        response.with_msg("ok");
        Ok(Json(response))
    }

    #[oai(path = "/filedownload", method = "get")]
    pub async fn filedownload(
        &self,
        app: Data<&AppState>,
        Query(req): Query<types::FileAction>
    ) -> Result<Attachment<Vec<u8>>> {

        let result = models::prelude::FileResult::find_by_id(req.id).one(&app.conn)
            .await
            .map_err(|e| {
                log::error!("Failed to filedownload: {}", e);
                Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                Error::from_string("File not found", StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        let path = format!("./data/{}",result.id);
        let data = fs::read(path)
            .await
            .map_err(|e| {
                log::error!("Failed to fileread: {}", e);
                Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        let mut attachment = Attachment::<Vec<u8>>::new(data).attachment_type(AttachmentType::Attachment);
        attachment = attachment.filename(result.filename);
        Ok(attachment)
    }

    #[oai(path = "/fileupload", method = "post")]
    pub async fn fileupload(
        &self,
        app: Data<&AppState>,
        mut multipart: Multipart
    ) -> Result<JsonResponse<Vec<models::fileresult::Model>>> {

        let mut response = ApiResponse::default();
        let mut result = Vec::new();

        while let Some(field) = multipart.next_field().await? {
            let filename = field.file_name().map(|s| s.to_string()).unwrap_or_default();
            let data = field.bytes().await?;

            let uuid = utils::uuid();
            let target_file = format!("./data/{}",uuid);
            let mut tmp_file = unwrap_err!(fs::File::create(target_file).await);
            unwrap_err!(tmp_file.write_all(&data).await);

            let mut active = models::fileresult::ActiveModel{
                id          : Set(uuid.clone()),
                target      : Set("NPU_UPLOAD".to_string()),
                agent       : Set("NPU_UPLOAD".to_string()),
                fileauth    : Set(utils::uuid()),
                filename    : Set(filename),
                filepath    : Set(uuid),
                filesize    : Set(data.len() as i64),
                .. Default::default()
            };

            let ret = unwrap_err!(active.insert(&app.conn).await);
            result.push(ret);
        }

        response.with_data(result);
        Ok(Json(response))
    }

}


