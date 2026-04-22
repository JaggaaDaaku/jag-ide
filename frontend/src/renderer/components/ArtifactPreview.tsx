import React from 'react';
import { useAgentStore } from '../stores/agentStore';

export const ArtifactPreview: React.FC = () => {
  const { recentArtifacts, approveArtifact, rejectArtifact } = useAgentStore();

  if (recentArtifacts.length === 0) {
    return (
      <div className="artifact-preview--empty">
        No artifacts generated yet. They will appear here as the agents work.
      </div>
    );
  }

  return (
    <div className="artifact-list">
      {recentArtifacts.map((art) => (
        <div key={art.id} className="artifact-item">
          <header className="artifact-item__header">
            <span className="artifact-item__type">{art.artifact_type}</span>
            <span className="artifact-item__time">
              {new Date(art.timestamp).toLocaleTimeString()}
            </span>
          </header>
          
          <div className="artifact-item__body">
            <div className="artifact-item__meta">
              <span>Agent: {art.created_by}</span>
              <span>Format: {art.format}</span>
              <span>Size: {(art.size / 1024).toFixed(1)} KB</span>
            </div>
            <div className="artifact-item__status">
              Status: <span className={`status--${art.verification_status.toLowerCase()}`}>
                {art.verification_status}
              </span>
            </div>
          </div>

          <footer className="artifact-item__footer">
            {art.verification_status === 'Pending' && (
              <div className="artifact-item__actions">
                <button 
                  className="btn--approve"
                  onClick={() => approveArtifact(art.id)}
                >
                  Approve ✅
                </button>
                <button 
                  className="btn--reject"
                  onClick={() => rejectArtifact(art.id)}
                >
                  Reject ❌
                </button>
              </div>
            )}
            <button className="btn--view">View Content 👁️</button>
          </footer>
        </div>
      ))}
    </div>
  );
};
