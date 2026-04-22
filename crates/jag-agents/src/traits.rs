use async_trait::async_trait;
use jag_core::types::*;
use jag_core::errors::Result;
use jag_workflow::engine::AgentExecutor;

/// Agent trait for Jag IDE specialized agents.
///
/// Each agent has a specific role (Planner, Backend, Frontend, Integration)
/// and handles a set of task types. Agents communicate via the A2AMessageBus.
///
/// This trait extends `AgentExecutor` (defined in jag-workflow to avoid
/// circular dependencies) with additional lifecycle methods.
#[async_trait]
pub trait Agent: AgentExecutor {
    /// Unique identifier for this agent instance.
    fn id(&self) -> AgentId;

    /// Handle an incoming inter-agent message.
    async fn on_message(&self, message: AgentMessage) -> Result<()>;

    /// Get the agent's current state.
    fn state(&self) -> AgentState;
}
