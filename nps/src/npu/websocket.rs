use crate::models;
use crate::protos;
use crate::core;
use crate::npu::user::types::UserInfo;

use super::{ApiResponse,JsonResponse};
use crate::unwrap_err;

use poem::web::websocket::{WebSocket,Message};
use poem::web::{Path,Query,Data,Multipart};
use poem::{
    Result,
    Error,
    handler,
    FromRequest,
    OnUpgrade,
    Response,
    Request,
    IntoResponse,
    http::StatusCode,
    session::Session,
};
use poem_openapi::{
    OpenApi, 
    payload::{Json,Attachment,AttachmentType}
};
use prost::Message as PbMessage;
use sea_orm::{EntityTrait,ActiveModelTrait,Set};
use futures::{SinkExt, StreamExt, Stream};
use tokio::io::{AsyncReadExt,AsyncWriteExt};

use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use std::io::Write;



#[handler]
pub async fn agentws( //固定id
    sess: &Session,
    ws: WebSocket,
    app: Data<&core::AppState>,
) -> Result<impl IntoResponse> {
    if let Some(user) = sess.get::<UserInfo>("USER"){
        let app1 = app.clone();
        Ok(ws.on_upgrade(move |socket| async move {
            let (sink, mut stream) = socket.split();

            while let Some(Ok(message)) = stream.next().await {
                match message {
                    Message::Binary(data) => {//c->a
                        let mut cursor = std::io::Cursor::new(data);
                        if let Ok(mut event) = protos::nps::AgentEvent::decode(&mut cursor){
                            event.id = user.userid.clone();
                            //app1.add_event(event).await;
                        }
                    },
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        }))
    }else{
        Err(Error::from_string("unauth", StatusCode::UNAUTHORIZED))
    }
}

#[handler]
pub async fn webproxy( //随机id
    Path((action,aid)): Path<(String,String)>,
    sess: &Session,
    ws: WebSocket,
    app: Data<&core::AppState>,
) -> Result<impl IntoResponse> {
    if let Some(_user) = sess.get::<UserInfo>("USER"){
        let app1 = app.clone();

        Ok(ws.on_upgrade(move |socket| async move {
            let (mut sink, mut stream) = socket.split();
            match action.as_str() {
                "log" => {
                    log::info!("log start {}", aid);
                    app1.add_client(&aid,sink).await;
                    while let Some(Ok(message)) = stream.next().await {
                        match message {
                            Message::Binary(data) => {//c->a
                                let mut cursor = std::io::Cursor::new(data);
                                if let Ok(mut event) = protos::nps::AgentEvent::decode(&mut cursor){
                                    event.id = aid.clone();
                                    app1.add_event(event).await;
                                }
                            },
                            Message::Close(_) => break,
                            _ => {}
                        }
                    }
                    app1.remove_client(&aid).await;
                    log::info!("log close {}", aid);
                },
                "ssh" => {
                    log::info!("ssh start {}", aid);
                    let rssh = core::RsshSession::new(app1.clone());
                    if let Ok(mut ssh) = rssh.ssh(&aid).await{
                        loop {
                            tokio::select! {
                                Some(msg) = ssh.wait() => {
                                    log::debug!("channel msg {:?}",msg);
                                    match msg {
                                        russh::ChannelMsg::Data { ref data } => {
                                            let _ = sink.send(Message::Text(String::from_utf8_lossy(&data.to_vec()).to_string())).await;
                                        }
                                        russh::ChannelMsg::ExitStatus{ exit_status } => {
                                            let _ = ssh.close().await;
                                            break;
                                        }
                                        russh::ChannelMsg::Close{ .. } => {
                                            break;
                                        }
                                        _ => {}
                                    }
                                },
                                Some(Ok(message)) = stream.next() => {
                                    match message{
                                        Message::Binary(data) => {
                                            let mut cursor = std::io::Cursor::new(data);
                                            if let Ok(event) = protos::nps::WsMsg::decode(&mut cursor){
                                                log::debug!("ws_msg msg {:?}",event);
                                                match event.enumof {
                                                    Some(protos::nps::ws_msg::Enumof::Resize(e)) => {
                                                        let _ = ssh.window_change(e.cols as u32, e.rows as u32, 0, 0).await;
                                                    },
                                                    Some(protos::nps::ws_msg::Enumof::Data(e)) => {
                                                        let _ = ssh.data(e.data.as_ref()).await;
                                                    },
                                                    _ => {}
                                                }
                                            }
                                        },
                                        Message::Close(_) => {
                                            let _ = ssh.close().await;
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    log::info!("ssh close {}", aid);
                },
                "vnc"|"gdi" => {
                    log::info!("vnc start {}", aid);
                    let rssh = core::RsshSession::new(app1.clone());
                    if let Ok(mut vnc) = rssh.vnc(&aid).await{
                        loop {
                            tokio::select! {
                                Some(message) = vnc.wait() => {
                                    match message {
                                        russh::ChannelMsg::Data{ ref data } => {//c->a
                                            let _ = sink.send(Message::Binary(data.to_vec())).await;
                                        },
                                        russh::ChannelMsg::ExitStatus{ exit_status } => {
                                            let _ = vnc.close().await;
                                            break;
                                        }
                                        russh::ChannelMsg::Close{ .. } => {
                                            break;
                                        }
                                        _ => {}
                                    }
                                },
                                Some(Ok(message)) = stream.next() => {
                                    match message{
                                        Message::Binary(data) => {
                                            let _ = vnc.data(data.as_ref()).await;
                                        },
                                        Message::Close(_) => {
                                            let _ = vnc.close().await;
                                            break;
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    log::info!("vnc close {}", aid);
                },
                "rdp" => {
                    log::info!("rdp start {}", aid);
                    let rssh = core::RsshSession::new(app1.clone());
                    if let Ok(mut rdp) = rssh.rdp(&aid).await{
                        loop {
                            tokio::select! {
                                Some(message) = rdp.wait() => {
                                    match message {
                                        russh::ChannelMsg::Data{ ref data } => {//c->a
                                            let _ = sink.send(Message::Binary(data.to_vec())).await;
                                        },
                                        russh::ChannelMsg::ExitStatus{ exit_status } => {
                                            let _ = rdp.close().await;
                                            break;
                                        }
                                        russh::ChannelMsg::Close{ .. } => {
                                            break;
                                        }
                                        _ => {}
                                    }
                                },
                                Some(Ok(message)) = stream.next() => {
                                    match message{
                                        Message::Binary(data) => {
                                            let _ = rdp.data(data.as_ref()).await;
                                        },
                                        Message::Close(_) => {
                                            let _ = rdp.close().await;
                                            break;
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    log::info!("rdp close {}", aid);
                },
                _ => {}
            }
        }))
    }else{
        Err(Error::from_string("unauth", StatusCode::UNAUTHORIZED))
    }
}