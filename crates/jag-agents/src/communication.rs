use dashmap::DashMap;
use tokio::sync::broadcast;
use jag_core::types::{AgentId, AgentMessage};
use jag_core::errors::{JagError, Result};

pub struct A2AMessageBus {
    sender: broadcast::Sender<AgentMessage>,
    subscribers: DashMap<AgentId, ()>,
}

impl Default for A2AMessageBus {
    fn default() -> Self {
        Self::new()
    }
}

impl A2AMessageBus {
    /// Create bus with channel capacity of 1000 messages
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self {
            sender,
            subscribers: DashMap::new(),
        }
    }

    /// Subscribe an agent — returns a Receiver
    pub fn subscribe(&self, agent_id: AgentId) -> broadcast::Receiver<AgentMessage> {
        self.subscribers.insert(agent_id, ());
        self.sender.subscribe()
    }

    /// Send targeted message (only agent matching `to` field should process)
    pub fn send(&self, message: AgentMessage) -> Result<()> {
        if message.to.is_none() {
            return Err(JagError::Internal("Use broadcast() for messages without a specific target.".into()));
        }
        
        let target = message.to.as_ref().unwrap();
        if !self.subscribers.contains_key(target) {
            // In a real system, you might queue it, but for now we'll just fail if not subscribed
            return Err(JagError::CommunicationError(format!("Target agent {} not subscribed", target.0)));
        }

        // Send returns exactly how many receivers received it. 
        // If 0, it means no active receivers are connected to the channel.
        self.sender.send(message).map_err(|e| JagError::CommunicationError(e.to_string()))?;
        Ok(())
    }

    /// Broadcast to all (set `to = None`)
    pub fn broadcast(&self, message: AgentMessage) -> Result<()> {
        if message.to.is_some() {
            return Err(JagError::Internal("Broadcast messages should have `to = None`.".into()));
        }
        self.sender.send(message).map_err(|e| JagError::CommunicationError(e.to_string()))?;
        Ok(())
    }

    /// Check if an agent is subscribed
    pub fn is_subscribed(&self, agent_id: &AgentId) -> bool {
        self.subscribers.contains_key(agent_id)
    }

    /// Unsubscribe an agent
    pub fn unsubscribe(&self, agent_id: &AgentId) {
        self.subscribers.remove(agent_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jag_core::types::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_dummy_message(from: AgentId, to: Option<AgentId>) -> AgentMessage {
        AgentMessage {
            id: MessageId::new(),
            from,
            to,
            timestamp: Utc::now(),
            message_type: MessageType::ErrorReport("Test".into()),
            payload: MessagePayload {
                artifact_id: None,
                task_id: None,
                data: serde_json::json!({}),
                metadata: HashMap::new(),
            },
            priority: Priority::Normal,
            correlation_id: None,
        }
    }

    #[tokio::test]
    async fn test_subscribe_and_targeted_message() {
        let bus = A2AMessageBus::new();
        let agent1 = AgentId::new();
        let agent2 = AgentId::new();

        let mut rx1 = bus.subscribe(agent1.clone());
        let _rx2 = bus.subscribe(agent2.clone());

        let msg = create_dummy_message(agent2.clone(), Some(agent1.clone()));
        
        bus.send(msg.clone()).unwrap();

        // Agent1 should receive and process
        let received = rx1.recv().await.unwrap();
        assert_eq!(received.id, msg.id);
        assert_eq!(received.to, Some(agent1));
    }

    #[tokio::test]
    async fn test_agent_filter_targeted_message() {
        let bus = A2AMessageBus::new();
        let target_agent = AgentId::new();
        let observer_agent = AgentId::new();

        let _rx_target = bus.subscribe(target_agent.clone());
        let mut rx_observer = bus.subscribe(observer_agent.clone());

        let msg = create_dummy_message(AgentId::new(), Some(target_agent.clone()));
        
        bus.send(msg).unwrap();

        // Observer receives the message because broadcast channel shares everything,
        // BUT logic expects filtering. We will manually simulate the agent filtering it.
        let received = rx_observer.recv().await.unwrap();
        let should_process = received.to.is_none() || received.to == Some(observer_agent.clone());
        assert!(!should_process);
    }

    #[tokio::test]
    async fn test_broadcast_reaches_all() {
        let bus = A2AMessageBus::new();
        let agent1 = AgentId::new();
        let agent2 = AgentId::new();

        let mut rx1 = bus.subscribe(agent1.clone());
        let mut rx2 = bus.subscribe(agent2.clone());

        let msg = create_dummy_message(AgentId::new(), None);
        bus.broadcast(msg.clone()).unwrap();

        let r1 = rx1.recv().await.unwrap();
        let r2 = rx2.recv().await.unwrap();

        assert_eq!(r1.id, msg.id);
        assert_eq!(r2.id, msg.id);
    }

    #[tokio::test]
    async fn test_channel_capacity_and_lagged() {
        let bus = A2AMessageBus::new();
        let agent1 = AgentId::new();
        let mut rx1 = bus.subscribe(agent1.clone());

        // Send 1050 messages, which exceeds 1000 capacity
        for _ in 0..1050 {
            let msg = create_dummy_message(AgentId::new(), None);
            bus.broadcast(msg).unwrap();
        }

        // Recv the first one, it will be skipped due to lagging
        let err = rx1.recv().await.unwrap_err();
        match err {
            broadcast::error::RecvError::Lagged(skipped) => {
                assert!(skipped > 0);
            }
            _ => panic!("Expected lagged error"),
        }

        // Once read again, it will catch up
        let msg = rx1.recv().await;
        assert!(msg.is_ok());
    }

    #[tokio::test]
    async fn test_unsubscribe_removes_agent() {
        let bus = A2AMessageBus::new();
        let agent1 = AgentId::new();
        
        let _rx1 = bus.subscribe(agent1.clone());
        assert!(bus.is_subscribed(&agent1));
        
        bus.unsubscribe(&agent1);
        assert!(!bus.is_subscribed(&agent1));

        let msg = create_dummy_message(AgentId::new(), Some(agent1.clone()));
        // Send targeted message to disconnected agent
        let res = bus.send(msg);
        assert!(res.is_err());
    }
}
