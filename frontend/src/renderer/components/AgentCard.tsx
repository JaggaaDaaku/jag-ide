import React from 'react';

interface AgentCardProps {
  role: string;
  status: string;
  progress: number;
  currentTask: string | null;
}

export const AgentCard: React.FC<AgentCardProps> = ({ role, status, progress, currentTask }) => {
  const getStatusColor = () => {
    switch (status.toLowerCase()) {
      case 'working': return 'var(--blue)';
      case 'completed': return 'var(--green)';
      case 'error': return 'var(--red)';
      default: return 'var(--overlay0)';
    }
  };

  const getRoleIcon = () => {
    switch (role.toLowerCase()) {
      case 'planner': return '🧠';
      case 'backend': return '⚙️';
      case 'frontend': return '🎨';
      case 'integration': return '🔗';
      default: return '🤖';
    }
  };

  return (
    <div className={`agent-card agent-card--${role.toLowerCase()}`}>
      <div className="agent-card__header">
        <span className="agent-card__icon">{getRoleIcon()}</span>
        <h3 className="agent-card__role">{role}</h3>
        <span className="agent-card__status-badge" style={{ backgroundColor: getStatusColor() }}>
          {status}
        </span>
      </div>
      
      <div className="agent-card__body">
        <div className="agent-card__progress-container">
          <div 
            className="agent-card__progress-bar" 
            style={{ width: `${progress}%`, backgroundColor: getStatusColor() }}
          />
        </div>
        <div className="agent-card__task">
          {currentTask ? `Task: ${currentTask}` : 'Waiting for tasks...'}
        </div>
      </div>
      
      <footer className="agent-card__footer">
        <span className="agent-card__model">Model: qwen2.5:7b</span>
      </footer>
    </div>
  );
};
