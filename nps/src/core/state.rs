
use crate::models;
use crate::protos;
use crate::npu::proxy::types::ProxyAction;
use crypto::AesCrypt;

use std::sync::Arc;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::{oneshot,mpsc,Mutex};
use prost::Message as PbMessage;
use serde::{Deserialize, Serialize};
use poem_openapi::Object;
use poem::web::websocket::{Message, WebSocketStream};
use sea_orm::{DatabaseConnection, ConnectOptions, Database, Set};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait};
use futures::SinkExt;
use futures::stream::{SplitSink, SplitStream};
use tokio_yamux::Control as YamuxControl;

#[derive(Clone, Default)]
pub struct NpsAgent {
    pub ntype: String,
    pub npc1: protos::nps::Agent,
    pub npc2: Option<YamuxControl>,
    pub ctrl: Option<mpsc::Sender<()>>,
}

impl NpsAgent {
    pub fn set_npc2(
        &mut self, 
        npc2: YamuxControl, 
        ctrl: mpsc::Sender<()>
    ) -> &mut Self {
        self.npc2 = Some(npc2);
        self.ctrl = Some(ctrl);
        self
    }
}

pub struct NpsProxy {
    pub ctrl: oneshot::Sender<()>,
    pub proxy: ProxyAction,
}


#[derive(Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub projects:   Arc<Mutex<HashMap<String, models::projects::Model>>>, //项目表
    pub events:     Arc<Mutex<HashMap<String, VecDeque<protos::nps::AgentEvent>>>>, //事务队列
    pub agents:     Arc<Mutex<HashMap<String, NpsAgent>>>, //npcs
    pub clients:    Arc<Mutex<HashMap<String, Box<SplitSink<WebSocketStream, Message>>>>>, //客户端
    pub listens:    Arc<Mutex<HashMap<String, oneshot::Sender<()>>>>, //监听器
    pub proxys:     Arc<Mutex<HashMap<u32,NpsProxy>>>, //代理
}

impl AppState {    

    pub async fn connect(datastr: &str) -> anyhow::Result<AppState> {
        let mut opts = ConnectOptions::new(datastr);
                opts.sqlx_logging(false); // Disable SQLx log
        let conn = Database::connect(opts).await?;

        Ok(AppState {
            conn, 
            projects:   Arc::new(Mutex::new(HashMap::new())),
            events:     Arc::new(Mutex::new(HashMap::new())),
            agents:     Arc::new(Mutex::new(HashMap::new())),
            clients:    Arc::new(Mutex::new(HashMap::new())),
            listens:    Arc::new(Mutex::new(HashMap::new())),
            proxys:     Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
///////////////////////////////////////////////////////////////////////////////////////////////

    pub async fn init(&self) -> anyhow::Result<()> {
        let active_projects = models::projects::Entity::find()
            .filter(models::projects::Column::Isrun.eq(true))
            .all(&self.conn)
            .await?;

        let mut projects = HashMap::new();
        for project in active_projects {
            projects.insert(project.route.clone(), project);
        }
        
        *self.projects.lock().await = projects;

        Ok(())
    }

    pub async fn is_projects_key(&self, route: &str) -> bool {
        self.projects
            .lock()
            .await
            .contains_key(route)
    }

    pub async fn get_project_by_route(&self, route: &str) -> Option<models::projects::Model> {
        self.projects
            .lock()
            .await
            .get(route)
            .cloned()
    }

    pub async fn project_active(&self, project: models::projects::Model) -> anyhow::Result<()> {
        let mut projects = self.projects.lock().await;

        if !project.isrun && projects.contains_key(&project.route) {
            projects.remove(&project.route);
            self.events
                .lock().await
                .remove(&project.route);
        }else{
            projects.insert(project.route.clone(), project);
        };
        Ok(())
    }
//////////////////////////////////////////////////////////////////////////////////////////////
    pub async fn add_event(&self, event: protos::nps::AgentEvent) {
        self.events
            .lock()
            .await
            .entry(event.sendto.clone())
            .or_insert(VecDeque::new())
            .push_back(event);
    }

    pub async fn get_event(&self, key: &str) -> Option<protos::nps::AgentEvent>{
        let mut ret = None;
        self.events
            .lock()
            .await
            .entry(key.to_string())
            .and_modify(|jobs| {
                ret = jobs.pop_front();
            })
            .or_insert(VecDeque::new());
        ret
    }
////////////////////////////////////////////////////////////////////////////////////////////
    pub async fn get_agent(&self, key: &str) -> Option<NpsAgent> {
        self.agents
            .lock()
            .await
            .get(key)
            .cloned()
    }

    pub async fn get_agents(&self) -> Vec<protos::nps::Agent> {
        self.agents
            .lock()
            .await
            .values()
            .map(|agent| {
                let mut npc = agent.npc1.clone();
                npc.npc2 = agent.npc2.is_some();
                if npc.npc2 {
                    npc.updateat = utils::timestamp();
                }
                npc
            })
            .collect()
    }

    pub async fn add_agent(&self, npc1: protos::nps::Agent) -> anyhow::Result<()> {
        let agent = NpsAgent{
            ntype: "npc1".to_string(),
            npc1: npc1.clone(),
            .. Default::default()
        };
        self.agents
            .lock()
            .await
            .insert(agent.npc1.id.clone(),agent);

        Ok(())
    }

    pub async fn add_agent2(
        &self, 
        npc1: protos::nps::Agent, 
        npc2: YamuxControl,
        ctrl: mpsc::Sender<()>
    ) -> anyhow::Result<()> {
        let insert = if let Some(agent) = self.agents.lock().await.get_mut(&npc1.id){
            agent.set_npc2(npc2.clone(),ctrl.clone());
            false
        }else{
            true
        };

        if insert {
            let agent = NpsAgent{
                ntype: "npc2".to_string(),
                npc1: npc1.clone(),
                npc2: Some(npc2),
                ctrl: Some(ctrl),
                .. Default::default()
            };
            self.agents
                .lock()
                .await
                .insert(agent.npc1.id.clone(),agent);
        }
        Ok(())
    }
    pub async fn remove_agent(&self, key: &str) {
        if let Some(agent) = self.agents.lock().await.remove(key){
            if let Some(ctrl) = agent.ctrl {
                let _ = ctrl.send(()).await;
            }
        }
    }
    pub async fn remove_agent2(&self, key: &str) {
        if let Some(agent) = self.agents.lock().await.get_mut(key){
            if let Some(ctrl) = &agent.ctrl {
                let _ = ctrl.send(()).await;
            } 
            agent.npc2 = None;
        }
    }

    pub async fn get_agent2(&self, key: &str) -> anyhow::Result<YamuxControl> {
        Ok(self.agents.lock().await
            .get(key)
            .ok_or(anyhow::anyhow!("Agent not found: {}", key))?
            .npc2
            .as_ref()
            .ok_or(anyhow::anyhow!("NPC2 handle not available for agent: {}", key))?
            .clone()
        )
    }

    pub async fn agent_note(&self, key: &str, note: String) -> anyhow::Result<()> {
        if let Some(agent) = self.agents.lock().await.get_mut(key){
            agent.npc1.note = note.clone();
        }
        Ok(())
    }

    pub async fn agent_heartbeat(&self, key: &str) -> bool {
        let mut ret = true;
        if let Some(agent) = self.agents.lock().await.get_mut(key){
            if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH){
                agent.npc1.updateat = now.as_secs() as i64;
            }
        }else{
            ret = false;
        }
        ret
    }


////////////////////////////////////////////////////////////////////////////////////////////////
    pub async fn add_client(&self, key: &str, sink: SplitSink<WebSocketStream, Message>){
        self.clients 
            .lock()
            .await
            .insert(key.to_string(), Box::new(sink));
    }

    pub async fn remove_client(&self, key: &str){
        self.clients 
            .lock()
            .await
            .remove(key);
    }
////////////////////////////////////////////////////////////////////////////////////////////////
    pub async fn send_msg(&self, event: protos::nps::AgentEvent) -> anyhow::Result<()> {
        if let Some(sink) = self.clients.lock().await.get_mut(&event.sendto){
            let mut buf = Vec::new();
            event.encode(&mut buf)?;
            sink.send(Message::Binary(buf)).await?;
        }

        Ok(())
    }

    pub async fn broadcast(&self, mut event: protos::nps::AgentEvent) -> anyhow::Result<()> {
        for (id, mut sink) in self.clients.lock().await.iter_mut() {
            let mut buf = Vec::new();
            event.sendto = id.to_string();
            event.encode(&mut buf)?;
            let msg = Message::Binary(buf);
            sink.send(msg).await?;
        }

        Ok(())
    }

///////////////////////////////////////////////////////////////////////////////////////////////

    pub async fn add_proxy(&self, ctrl: oneshot::Sender<()>, proxy: ProxyAction){

        let sp = NpsProxy{ctrl,proxy};
        self.proxys
            .lock()
            .await
            .insert(sp.proxy.local_port,sp);

    }

    pub async fn get_proxys(&self) -> Vec<ProxyAction> {
        self.proxys
            .lock()
            .await
            .values()
            .map(|sp| {
                sp.proxy.clone()
            })
            .collect()
    }

    pub async fn stop_proxy(&self, id: &str) -> anyhow::Result<()> {
        let mut proxys = self.proxys.lock().await;
        if let Some((&port, _)) = proxys.iter().find(|(_, sp)| sp.proxy.id == id) {
            if let Some(sp) = proxys.remove(&port) {
                sp.ctrl.send(()).map_err(|_| anyhow::anyhow!("Failed to send stop proxy"))?;
                log::info!("stop proxy {}",id);
            }
        }
        Ok(())
    }

///////////////////////////////////////////////////////////////

    pub async fn add_listen(&self, id: &str, ctrl: oneshot::Sender<()>) {
        self.listens.lock().await.insert(id.to_string(),ctrl);
    }

    pub async fn stop_listen(&self, id: &str) -> anyhow::Result<()> {
        let mut listens = self.listens.lock().await;
        if let Some(ctrl) = listens.remove(id) {
            ctrl.send(()).map_err(|_| anyhow::anyhow!("Failed to send stop listen"))?;
            log::info!("stop listen {}",id);
        }
        Ok(())
    }

}