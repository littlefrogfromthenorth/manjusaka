pub mod user;
pub mod project;
pub mod listen;
pub mod setting;
pub mod agent;
pub mod fileresult;
pub mod passresult;
pub mod proxy;
pub mod websocket;


use crate::core;
use crate::models;


use std::io::Read;
use std::time::Duration;

use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use poem_openapi::{
    payload::Json,
    types::{ParseFromJSON, ToJSON},
    Object,
    OpenApiService
};

use poem::{
    get,
    handler,
    Endpoint,
    IntoResponse, 
    Middleware, 
    Request, 
    Response, 
    Result,
    web::Redirect,
    endpoint::{EmbeddedFileEndpoint, EmbeddedFilesEndpoint},   
    listener::TcpListener,
    session::{Session, ServerSession,CookieConfig,MemoryStorage},
    Route,
    EndpointExt
};
use poem::listener::{Listener, RustlsCertificate, RustlsConfig};


#[derive(Object, Serialize, Deserialize)]
pub struct ApiResponse<T: ParseFromJSON + ToJSON> {
    pub code: u16,
    pub message: String,
    pub result: Option<T>,
}

impl<T> Default for ApiResponse<T>
where
    T: ParseFromJSON + ToJSON 
{
    fn default() -> Self {
        ApiResponse{
            code: 200,
            message: String::from("suss"),
            result: None
        }
    }
}

impl<T> ApiResponse<T>
where
    T: ParseFromJSON + ToJSON
{
    pub fn with_code(&mut self, code: u16) -> &mut Self {
        self.code = code;
        self
    }

    pub fn with_msg(&mut self, msg: impl Into<String>) -> &mut Self {
        self.message = msg.into();
        self
    }

    pub fn with_data(&mut self, data: T) -> &mut Self {
        self.result = Some(data);
        self
    }
}


pub type JsonResponse<T> = Json<ApiResponse<T>>;


pub struct AuthMiddleware;

impl<E: Endpoint> Middleware<E> for AuthMiddleware {
    type Output = AuthMiddlewareEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        AuthMiddlewareEndpoint { ep }
    }
}

pub struct AuthMiddlewareEndpoint<E> {
    ep: E,
}

// #[async_trait::async_trait]
impl<E> Endpoint for AuthMiddlewareEndpoint<E>
where
    E: Endpoint,
{
    type Output = Response;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        let resp = Json(serde_json::json! ({
            "code": 401,
            "msg": "not login",
        })).into_response();

        let sess: &Session = req.extensions().get().expect("not init session");

        if let Some(user_info) = sess.get::<user::types::UserInfo>("USER") {
            req.extensions_mut().insert(user_info);
        } else {
            if vec!["/captcha", "/login"].contains(&req.uri().path()){
                return self.ep.call(req).await.map(IntoResponse::into_response);
            }
            return Ok(resp);
        }
        self.ep.call(req).await.map(IntoResponse::into_response)
    }
}



#[derive(RustEmbed)]
#[folder = "../npu/dist"]
pub struct Dist;

pub async fn start(addr: &str, app: core::AppState) -> Result<(), Box<dyn std::error::Error>> {
    let api_service = OpenApiService::new(
        (
            user::UserApi, 
            listen::ListenApi,
            project::ProjectApi, 
            agent::AgentApi,
            fileresult::FileApi,
            passresult::PassApi,
            setting::SettingApi,
            proxy::ProxyApi,
        ),
        "Manjusaka", 
        "1.0"
    ).server(addr);

    let rapidoc = api_service.rapidoc();
    let manjusaka_router = Route::new()
        .at("/webproxy/:action/:aid", get(websocket::webproxy).with(AuthMiddleware))
        .at("/agentws", get(websocket::agentws).with(AuthMiddleware))
        //.at("/", EmbeddedFileEndpoint::<Dist>::new("index.html"))
        //.nest("/rapidoc/", rapidoc)
        .nest("/static/", EmbeddedFilesEndpoint::<Dist>::new())
        .nest("/", api_service.with(AuthMiddleware));

    let admin = models::settings::get(&app.conn,"nps.admin","manjusaka").await;
    let npsapp = Route::new()
        .nest(admin, manjusaka_router)
        .with(ServerSession::new(
                CookieConfig::default()
                    .name("AUTH")
                    .secure(false) 
                    .http_only(true)
                    .max_age(Some(Duration::from_secs(3600 * 24))),
                MemoryStorage::new()))
        .data(app);

    //let crt_bytes = std::fs::read("certs/server.crt")?;
    //let key_bytes = std::fs::read("certs/server.key")?;
    //let config = RustlsConfig::new().fallback(RustlsCertificate::new().cert(crt_bytes).key(key_bytes));
    let listener = TcpListener::bind(addr);

    Ok(poem::Server::new(listener).run(npsapp).await?)
}