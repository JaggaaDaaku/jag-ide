import React, { useEffect, useState } from 'react';
import { MissionControl } from './components/MissionControl';
import { ErrorBoundary } from './components/ErrorBoundary';

/**
 * App — Top-level shell component.
 * Renders the IDE chrome: titlebar, activity bar, panels, status bar.
 * UI transitions between EditorView ↔ MissionControl via Ctrl+Shift+M.
 */
const App: React.FC = () => {
  const [showMissionControl, setShowMissionControl] = useState(false);

  // Keyboard shortcut: Ctrl+Shift+M → toggle Mission Control
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === 'M') {
        e.preventDefault();
        setShowMissionControl(prev => !prev);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  return (
    <ErrorBoundary>
      <div className={`app-shell ${showMissionControl ? 'app-shell--mission' : ''}`}>
      {/* Custom Titlebar (zero-chrome) */}
      <header className="title-bar" data-electron-drag>
        <div className="title-bar__left">
          <span className="title-bar__logo">⬡</span>
          <span className="title-bar__name">Jag IDE {showMissionControl && '— Mission Control'}</span>
        </div>
        <div className="title-bar__controls">
          <button
            className="title-btn title-btn--min"
            onClick={() => (window as any).jagBridge?.window.minimize()}
            title="Minimize"
          >
            ─
          </button>
          <button
            className="title-btn title-btn--max"
            onClick={() => (window as any).jagBridge?.window.maximize()}
            title="Maximize"
          >
            □
          </button>
          <button
            className="title-btn title-btn--close"
            onClick={() => (window as any).jagBridge?.window.close()}
            title="Close"
          >
            ✕
          </button>
        </div>
      </header>

      {/* Main Container */}
      <div className="main-container">
        {showMissionControl ? (
          <MissionControl />
        ) : (
          <div className="workspace">
            {/* Activity Bar — left icon rail */}
            <aside className="activity-bar" role="navigation" aria-label="Activity">
              <button className="activity-btn active" data-tooltip="Explorer">📁</button>
              <button className="activity-btn" data-tooltip="Search">🔍</button>
              <button className="activity-btn" data-tooltip="Source Control">🌿</button>
              <button className="activity-btn" data-tooltip="Run & Debug">▶️</button>
              <button className="activity-btn" data-tooltip="Extensions">🧩</button>
              <div className="activity-bar__spacer" />
              <button 
                className="activity-btn activity-btn--agents" 
                data-tooltip="Agent Control"
                onClick={() => setShowMissionControl(true)}
              >
                🤖
              </button>
            </aside>

            {/* Primary Side Panel */}
            <aside className="primary-panel">
              <div className="panel-header">
                <span className="panel-title">EXPLORER</span>
              </div>
              <div className="panel-body">
                <p className="panel-empty">Open a folder to get started</p>
              </div>
            </aside>

            {/* Editor Surface */}
            <main className="editor-surface">
              <div className="editor-placeholder">
                <div className="welcome-screen">
                  <div className="welcome-logo">⬡</div>
                  <h1 className="welcome-title">Jag IDE</h1>
                  <p className="welcome-subtitle">Agent-first autonomous development</p>
                  <div className="welcome-actions">
                    <button 
                      className="welcome-btn welcome-btn--primary"
                      onClick={() => setShowMissionControl(true)}
                    >
                      🤖 &nbsp; Start Mission Control
                    </button>
                    <button 
                      className="welcome-btn"
                      onClick={async () => {
                        const path = await (window as any).jagBridge?.window.openFolder();
                        if (path) console.log('Opened folder:', path);
                      }}
                    >
                      📁 &nbsp; Open Folder
                    </button>
                    <button 
                      className="welcome-btn"
                      onClick={async () => {
                        const path = await (window as any).jagBridge?.window.newFile();
                        if (path) console.log('New file:', path);
                      }}
                    >
                      📄 &nbsp; New File
                    </button>
                  </div>
                  <p className="welcome-shortcut">
                    Press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>M</kbd> to toggle Mission Control
                  </p>
                </div>
              </div>
            </main>

            {/* Secondary Panel */}
            <aside className="secondary-panel">
              <div className="panel-header">
                <button className="panel-tab active">Chat</button>
                <button className="panel-tab">Artifacts</button>
                <button className="panel-tab">Terminal</button>
              </div>
              <div className="panel-body">
                <div className="chat-placeholder">
                  <span className="chat-icon">🤖</span>
                  <p>Ask an agent to start working…</p>
                </div>
              </div>
            </aside>
          </div>
        )}
      </div>

      {/* Status Bar */}
      <footer className="status-bar">
        <span className="status-item status-item--branch">🌿 main</span>
        <span className="status-item">Ln 1, Col 1</span>
        <span className="status-item">UTF-8</span>
        <div className="status-bar__spacer" />
        <span className="status-item status-item--agents" onClick={() => setShowMissionControl(true)} style={{cursor: 'pointer'}} data-tooltip="Toggle Mission Control">
          🤖 0 active agents
        </span>
        <span className="status-item status-item--tier" data-tooltip="Security Tier (Configurable in Settings)">🔒 Tier: Auto</span>
      </footer>
    </div>
    </ErrorBoundary>
  );
};

export default App;

