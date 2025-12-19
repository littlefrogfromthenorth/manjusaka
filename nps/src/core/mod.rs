
mod state;
mod rssh;
mod srdi;

pub use state::AppState;
pub use rssh::{RsshClient,RsshSession};
use crate::protos;

use tokio::time::{interval, sleep, Duration};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn start(app: AppState) {
	tokio::spawn(async move{
		let wait: i64 = 50;
		let mut monitor = interval(Duration::from_secs(wait as u64));
		loop {
			monitor.tick().await;
			let mut agents = Vec::new();
			if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH){
                for agent in app.get_agents().await {
					if now.as_secs() as i64 - agent.updateat < wait {
						agents.push(agent);
					}
				}
            }
			
			if !agents.is_empty() {
				let mut agentlist = protos::nps::AgentList::default();
	            agentlist.status = "update".to_string();
	            agentlist.agents = agents;
	            let mut agentevent = protos::nps::AgentEvent::default();
	            agentevent.enumof =  Some(protos::nps::agent_event::Enumof::List(agentlist));
				let _ = app.broadcast(agentevent).await;
			}
		}
	});
}