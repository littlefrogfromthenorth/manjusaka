use crate::core;
use crate::protos;
use crate::models;
use noise::NoiseConfig;

use std::convert::TryFrom;
use std::time::Duration;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use futures::StreamExt;
use tokio::time;
use tokio::sync::{oneshot,mpsc};
use tokio::net::TcpListener;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncReadExt,AsyncWriteExt};
use serde::{Serialize, Deserialize};
use prost::Message;
use tokio_yamux::{
    session::SessionType as YamuxMode, 
    Config as YamuxConfig, 
    Session as YamuxSession,
    Control as YamuxControl,
};


async fn handshake<S>(mut conn: S) -> anyhow::Result<protos::npc2::Agent>
	where S: AsyncRead + AsyncWrite + Unpin {

    let mut length = [0u8; 4];
    let _ = conn.read_exact(&mut length).await?;
    let length_value = u32::from_be_bytes(length);
    let mut data = Vec::new();
    let mut buffer = [0u8; 1024];
    let mut total_read = 0;
    while total_read < length_value as usize {
        let bytes_read = conn.read(&mut buffer).await?;
        data.extend_from_slice(&buffer[..bytes_read]);
        total_read += bytes_read;
    }
    let mut cursor = std::io::Cursor::new(data);
    let event = protos::npc2::Agent::decode(&mut cursor)?;
    
    Ok(event)
}


pub async fn start(app: core::AppState) -> anyhow::Result<()> {

    let active_listens = models::listens::Entity::find()
        .filter(models::listens::Column::Isrun.eq(true))
        .all(&app.conn)
        .await?;

    for listen in active_listens {
    	let app1 = app.clone();
		if let Err(e) = start_listen(app1, listen).await{
			log::error!("start_listen {}",e);
		}
    }

	Ok(())
}

pub async fn start_listen(app: core::AppState, listen: models::listens::Model) -> anyhow::Result<()> {
	let noise = NoiseConfig::new(listen.enckey.as_bytes(),&listen.noise)?;
	
	//tcp
	let socket = TcpListener::bind(&listen.listenaddr).await?;


	log::info!("start listen bind on {}",listen.listenaddr);

	let (tx, mut rx) = oneshot::channel();
	app.add_listen(&listen.id, tx).await;
	tokio::spawn(async move{
		if let Err(e) = npc2_handler(socket, app, noise, rx).await{
			log::error!("npc2_handler {}",e);
		}
	});

	Ok(())
}

async fn npc2_handler(
	socket:TcpListener, 
	app: core::AppState, 
	noise: NoiseConfig, 
	mut rx: oneshot::Receiver<()>
) -> anyhow::Result<()> {
	loop{
		tokio::select!{
			_ = &mut rx => {
	            break
	        },
			Ok((mut stream, addr)) = socket.accept() => {
		    	let app1 = app.clone(); 
		    	let noise1 = noise.clone(); 

			    tokio::spawn(async move {
			    	if let Ok(Ok(agent)) = time::timeout(Duration::from_secs(5),
			    		noise1.responder(&mut stream)).await {
			        	if let Ok(Ok(agent)) = time::timeout(Duration::from_secs(5),
			        		handshake(&mut stream)).await {
			        		let mut nps_agent 	   = protos::nps::Agent::default();
								nps_agent.id 	   = agent.id.clone();
							    nps_agent.intranet = agent.intranet.clone();
							    nps_agent.username = agent.username.clone();
							    nps_agent.hostname = agent.hostname.clone();
							    nps_agent.platform = agent.platform.clone();
							    nps_agent.arch 	   = agent.arch.clone();
							    nps_agent.process  = agent.process.clone();
							    nps_agent.pid 	   = agent.pid.clone();
							    
		                        nps_agent.updateat = utils::timestamp();
		                        nps_agent.internet = addr.to_string();
							    nps_agent.target   = "manjusaka".to_string();
		                        nps_agent.note 	   = "npc2上线".to_string();

			        		let mut session = YamuxSession::new(stream, YamuxConfig::default(), YamuxMode::Server);
			                let control = session.control();
			                log::info!("npc2 connect {}", agent.id);
			                let (tx, mut rx) = mpsc::channel(10);
			                let _ = app1.add_agent2(nps_agent,control,tx).await;
			                loop {
			                	tokio::select!{
			                		Some(e) = session.next() => { 
			                			//log::info!("{:?}",e);
						        		break;
						        	},
						        	_ = rx.recv() => {
				                        break
				                    },
						        }
						    }
						    //下线
						    app1.remove_agent2(&agent.id).await;
						    let _ = app1.stop_proxy(&agent.id).await;
						    log::info!("npc2 disconnect {}", agent.id);
				        }
			        }
			    });
			}
		}
	}
	Ok(())
}




