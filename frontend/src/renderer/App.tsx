import React, { useEffect } from 'react';

/**
 * App — Top-level shell component.
 * Renders the IDE chrome: titlebar, activity bar, panels, status bar.
 * UI transitions between EditorView ↔ ManagerSurface via Ctrl+Shift+M.
 */
const App: React.FC = () => {
  // Keyboard shortcut: Ctrl+Shift+M → toggle Mission Control
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === 'M') {
        e.preventDefault();
        // TODO: toggle layout store view (EditorView ↔ ManagerSurface)
        console.log('[Jag] Toggle Mission Control');
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  return (
    <div className="app-shell">
      {/* Custom Titlebar (zero-chrome) */}
      <header className="title-bar" data-electron-drag>
        <span className="title-bar__logo">⬡</span>
        <span className="title-bar__name">Jag IDE</span>
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

      {/* Main IDE Layout */}
      <div className="workspace">
        {/* Activity Bar — left icon rail */}
        <aside className="activity-bar" role="navigation" aria-label="Activity">
          <button className="activity-btn active" title="Explorer">📁</button>
          <button className="activity-btn" title="Search">🔍</button>
          <button className="activity-btn" title="Source Control">🌿</button>
          <button className="activity-btn" title="Run & Debug">▶️</button>
          <button className="activity-btn" title="Extensions">🧩</button>
          <div className="activity-bar__spacer" />
          <button className="activity-btn activity-btn--agents" title="Agent Control">🤖</button>
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

        {/* Editor + Mission Control surface */}
        <main className="editor-surface">
          <div className="editor-placeholder">
            <div className="welcome-screen">
              <div className="welcome-logo">⬡</div>
              <h1 className="welcome-title">Jag IDE</h1>
              <p className="welcome-subtitle">Agent-first autonomous development</p>
              <div className="welcome-actions">
                <button className="welcome-btn welcome-btn--primary">
                  🤖 &nbsp; Start Mission Control
                </button>
                <button className="welcome-btn">
                  📁 &nbsp; Open Folder
                </button>
                <button className="welcome-btn">
                  📄 &nbsp; New File
                </button>
              </div>
              <p className="welcome-shortcut">
                Press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>M</kbd> to toggle Mission Control
              </p>
            </div>
          </div>
        </main>

        {/* Secondary Panel — right side (chat, artifacts, terminal) */}
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

      {/* Status Bar */}
      <footer className="status-bar">
        <span className="status-item status-item--branch">🌿 main</span>
        <span className="status-item">Ln 1, Col 1</span>
        <span className="status-item">UTF-8</span>
        <div className="status-bar__spacer" />
        <span className="status-item status-item--agents">🤖 0 active agents</span>
        <span className="status-item status-item--tier">🔒 Tier: Auto</span>
      </footer>
    </div>
  );
};

export default App;
