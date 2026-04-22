import React, { useState, useEffect } from 'react';
import { useAgentStore } from '../stores/agentStore';
import { AgentCard } from './AgentCard';
import { WorkflowTimeline } from './WorkflowTimeline';
import { ArtifactPreview } from './ArtifactPreview';
import { AnalyticsDashboard } from './AnalyticsDashboard';
import { AuditLogViewer } from './AuditLogViewer';
import { ApprovalQueue } from './ApprovalQueue';
import { BrowserTestResults } from './BrowserTestResults';
import { PrStatus } from './PrStatus';
import { CoverageReport } from './CoverageReport';

export const MissionControl: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'mission' | 'analytics' | 'audit' | 'approvals' | 'visuals' | 'governance'>('mission');
  const { 
    agents, 
    workflow, 
    startPolling, 
    stopPolling, 
    startWorkflow,
    error 
  } = useAgentStore();
  
  const [description, setDescription] = useState('');

  useEffect(() => {
    startPolling();
    return () => stopPolling();
  }, []);

  const handleStart = () => {
    if (!description.trim()) return;
    startWorkflow(description);
  };

  return (
    <div className="mission-control">
      <header className="mission-control__header">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <h1>🚀 Jag Mission Control</h1>
          <div className="tab-nav" style={{ display: 'flex', gap: '10px' }}>
            <button 
              className={`tab-btn ${activeTab === 'mission' ? 'active' : ''}`}
              onClick={() => setActiveTab('mission')}
              style={{ padding: '8px 16px', borderRadius: '8px', border: '1px solid #ddd', background: activeTab === 'mission' ? '#000' : '#fff', color: activeTab === 'mission' ? '#fff' : '#000', cursor: 'pointer' }}
            >
              Mission
            </button>
            <button 
              className={`tab-btn ${activeTab === 'analytics' ? 'active' : ''}`}
              onClick={() => setActiveTab('analytics')}
              style={{ padding: '8px 16px', borderRadius: '8px', border: '1px solid #ddd', background: activeTab === 'analytics' ? '#000' : '#fff', color: activeTab === 'analytics' ? '#fff' : '#000', cursor: 'pointer' }}
            >
              Analytics
            </button>
            <button 
              className={`tab-btn ${activeTab === 'audit' ? 'active' : ''}`}
              onClick={() => setActiveTab('audit')}
              style={{ padding: '8px 16px', borderRadius: '8px', border: '1px solid #ddd', background: activeTab === 'audit' ? '#000' : '#fff', color: activeTab === 'audit' ? '#fff' : '#000', cursor: 'pointer' }}
            >
              Audit Log
            </button>
            <button 
              className={`tab-btn ${activeTab === 'approvals' ? 'active' : ''}`}
              onClick={() => setActiveTab('approvals')}
              style={{ padding: '8px 16px', borderRadius: '8px', border: '1px solid #ddd', background: activeTab === 'approvals' ? '#000' : '#fff', color: activeTab === 'approvals' ? '#fff' : '#000', cursor: 'pointer' }}
            >
              Approvals
            </button>
            <button 
              className={`tab-btn ${activeTab === 'visuals' ? 'active' : ''}`}
              onClick={() => setActiveTab('visuals')}
              style={{ padding: '8px 16px', borderRadius: '8px', border: '1px solid #ddd', background: activeTab === 'visuals' ? '#000' : '#fff', color: activeTab === 'visuals' ? '#fff' : '#000', cursor: 'pointer' }}
            >
              Visual Tests
            </button>
            <button 
              className={`tab-btn ${activeTab === 'governance' ? 'active' : ''}`}
              onClick={() => setActiveTab('governance')}
              style={{ padding: '8px 16px', borderRadius: '8px', border: '1px solid #ddd', background: activeTab === 'governance' ? '#000' : '#fff', color: activeTab === 'governance' ? '#fff' : '#000', cursor: 'pointer' }}
            >
              Governance
            </button>
          </div>
        </div>

        {activeTab === 'mission' && (
          <div className="mission-control__input-group" style={{ marginTop: '20px' }}>
            <input 
              type="text" 
              placeholder="Tell Jag what to build... (e.g., 'Build a secure todo app with Rust & React')" 
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleStart()}
            />
            <button onClick={handleStart} disabled={!description.trim()}>
              Start Mission
            </button>
          </div>
        )}
        {error && <div className="mission-control__error">Error: {error}</div>}
      </header>

      <main className="mission-control__content">
        {activeTab === 'mission' && (
          <>
            <section className="mission-control__agents">
              <h2>Team Status</h2>
              <div className="agent-grid">
                {['Planner', 'Backend', 'Frontend', 'Integration'].map(role => {
                  const agent = agents.find(a => a.role === role);
                  return (
                    <AgentCard 
                      key={role}
                      role={role}
                      status={agent?.status || 'Idle'}
                      progress={agent?.progress || 0}
                      currentTask={agent?.current_task || null}
                    />
                  );
                })}
              </div>
            </section>

            <section className="mission-control__workflow">
              <h2>Workflow Timeline</h2>
              <WorkflowTimeline workflow={workflow} />
            </section>

            <section className="mission-control__artifacts">
              <h2>Latest Artifacts</h2>
              <ArtifactPreview />
            </section>
          </>
        )}

        {activeTab === 'analytics' && (
          <section className="mission-control__analytics">
            <AnalyticsDashboard />
          </section>
        )}

        {activeTab === 'audit' && (
          <section className="mission-control__audit">
            <AuditLogViewer />
          </section>
        )}

        {activeTab === 'approvals' && (
          <section className="mission-control__approvals">
            <ApprovalQueue />
          </section>
        )}

        {activeTab === 'visuals' && (
          <section className="mission-control__visuals">
            <BrowserTestResults />
          </section>
        )}
        
        {activeTab === 'governance' && (
          <section className="mission-control__governance" style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
            <CoverageReport />
            <PrStatus />
          </section>
        )}
      </main>
    </div>
  );
};
