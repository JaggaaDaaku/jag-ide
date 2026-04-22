import React from 'react';

interface WorkflowTimelineProps {
  workflow: any; // Simplified for Phase 2
}

export const WorkflowTimeline: React.FC<WorkflowTimelineProps> = ({ workflow }) => {
  if (!workflow) {
    return <div className="workflow-timeline--empty">No active workflow. Start a mission to see the timeline.</div>;
  }

  // Simplified task list for Phase 2 demo
  const mockTasks = [
    { name: 'Generate PRD', status: 'Completed', id: '1' },
    { name: 'Design Architecture', status: 'Working', id: '2' },
    { name: 'Implement API', status: 'Pending', id: '3' },
    { name: 'Build Dashboard UI', status: 'Pending', id: '4' },
  ];

  return (
    <div className="workflow-timeline">
      {mockTasks.map((task, index) => (
        <div key={task.id} className={`workflow-node workflow-node--${task.status.toLowerCase()}`}>
          <div className="workflow-node__line" />
          <div className="workflow-node__marker">
             {task.status === 'Completed' ? '✅' : task.status === 'Working' ? '🔄' : '⏳'}
          </div>
          <div className="workflow-node__content">
            <span className="workflow-node__name">{task.name}</span>
            <span className="workflow-node__status">{task.status}</span>
          </div>
        </div>
      ))}
    </div>
  );
};
