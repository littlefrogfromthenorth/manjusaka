use super::{ApiResponse,JsonResponse};
use crate::unwrap_err;
use crate::models;
use crate::core;

use std::net::SocketAddr;
use std::str::FromStr;

use poem::{
    web::Data,
    http::StatusCode,
    session::Session,
    Error,
    Result
};

use poem_openapi::{
    OpenApi, 
    payload::Json,
};

pub mod types {
    use poem_openapi::Object;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Object, Deserialize, Serialize)]
    pub struct ProxyAction {
        pub id: String, 
        pub agent: Option<String>, 
        pub name: String,    
        pub local_port: u32,
        pub remote_addr: Option<String>,
        pub username: Option<String>,
        pub password: Option<String>,
        pub note: Option<String>,
    }

    #[derive(Debug, Object, Deserialize, Serialize)]
    pub struct ProxyId {
        pub id: String,
    }
}

pub struct ProxyApi;

#[OpenApi]
impl ProxyApi {
    #[oai(path = "/proxylist", method = "post")]
    pub async fn proxylist(
        &self,
        app: Data<&core::AppState>
    ) -> Result<JsonResponse<Vec<types::ProxyAction>>> {
        let mut response = ApiResponse::default();
        response.with_data(app.get_proxys().await);
        Ok(Json(response))
    }

    #[oai(path = "/proxyadd", method = "post")]
    pub async fn proxyadd(
        &self,
        app: Data<&core::AppState>,
        Json(req): Json<types::ProxyAction>
    ) -> Result<JsonResponse<String>> {
        
        let username = req.username.clone().unwrap_or("buut".to_string());
        let password = req.password.clone().unwrap_or("buut".to_string());
        
        let rssh = core::RsshSession::new(app.clone());

        let tx = if let Some(ref addr) = req.remote_addr { //
            let raddr: SocketAddr = unwrap_err!(addr.parse());

            log::info!("add mapping {}   lport:{} <---> rport:{:?} ",req.id,req.local_port,raddr);
            let tx = unwrap_err!(rssh.proxy(&req.id, req.local_port, Some(raddr), username,password).await);
            tx
        }else{
            log::info!("add proxy {}://{}:{}@0.0.0.0:{}/{} ",req.name,username,password,req.local_port,req.id);
            let tx = unwrap_err!(rssh.proxy(&req.id, req.local_port, None, username,password).await);
            tx
        };

        let agent = if let Some(a) = app.get_agent(&req.id).await{
            format!("{}@{}",a.npc1.username,a.npc1.intranet)
        }else{
            req.id.clone()
        };

        let proxy = types::ProxyAction{
            agent: Some(agent),
            .. req
        };

        app.add_proxy(tx,proxy).await;

        let mut response = ApiResponse::default();
        Ok(Json(response))
    }

    #[oai(path = "/proxystop", method = "post")]
    pub async fn proxystop(
        &self,
        app: Data<&core::AppState>,
        Json(req): Json<types::ProxyId>
    ) -> Result<JsonResponse<String>> {

        unwrap_err!(app.stop_proxy(&req.id).await);

        let mut response = ApiResponse::default();
        Ok(Json(response))
    }
}


