# 🧠 Antigravity UI/UX → Frontend Engineering Blueprint
`[REFERENCE]` Proceeding with inference based on publicly documented Antigravity UI patterns, Google I/O 2025/2026 demos, and comparative analysis with Zed, Cursor, Nova, and Lapce. All inferred elements are explicitly marked `[ASSUMPTION]`.

---

## 1. 🧩 CORE FEATURES & FUNCTIONS

| Feature Domain | Antigravity-Specific Implementation | Standard IDE Baseline |
|----------------|-------------------------------------|----------------------|
| **Code Editing** | Virtualized buffer + WebGPU gutter rendering. Ghost text for AI inline completions. Multi-cursor, bracket matching, virtual whitespace. | LSP-driven diagnostics, snippets, fold regions, minimap |
| **AI/Agent Core** | **Dual-Interface**: Editor View ↔ Manager Surface. Multi-agent dispatch, `Sticky Brain` model selector, comment-based artifact feedback, security tier toggles (Off/Auto/Turbo). | `[ASSUMPTION]` AI sidebar chat, inline commands |
| **Artifact Verification** | Visual deliverables: PRD markdown, Mermaid architecture, diff viewers, test matrices, browser screenshot galleries. No raw logs. | Output panel, console, problems tab |
| **Debugging & Profiling** | DAP integration. Breakpoint gutter, call stack, variables/watch, timeline profiler. AI-assisted trace analysis. | Standard DAP UI, step controls, conditional breakpoints |
| **Terminal & Shell** | PTY bridge (native). Multiplexing, theme sync, shell integration (prompt decoration, command suggestion, inline error hints). | xterm.js/WASM terminal, env var injection |
| **Version Control** | Git graph visualization, split diff/merge editor, blame gutter, commit flow, AI conflict resolution suggestions. | Git integration, rebase UI, stash manager |
| **Extension Ecosystem** | Marketplace UI, sandboxed webviews (iframe/COOP), permission prompts, update flow. Extension API supports AI tool registration. | Extension host, webview API, activation events |
| **Workspace Management** | Multi-root support, tab/split layout presets, session restore, cloud sync indicator. Agent workspaces isolated per project. | Folder tree, recent files, workspace trust |
| **Search & Navigation** | Global fuzzy search, symbol jump, command palette (`/agent`, `/explain`, `/generate`), AI semantic codebase search via embeddings. | File picker, symbol search, regex find/replace |

---

## 2. 📑 MENU STRUCTURE & NAVIGATION HIERARCHY

### Top-Level Menu Tree
```
File → New Workspace, Open, Open Recent, Save, Save As, Close, Exit
Edit → Undo/Redo, Cut/Copy/Paste, Find/Replace, Command Palette (Ctrl+Shift+P)
View → Explorer, Search, SCM, Run/Debug, Extensions, Agents 🤖, Manager 🎯, Terminal, Zoom
Agent → Create Team, Dispatch Task, Model Garden, Security Config, Active Agents
Run/Debug → Start (F5), Run Without Debugging, Stop, Restart, Run Task
Terminal → New Terminal, Split Terminal, Run Active File, Clear
Help → Welcome, Docs, Keyboard Shortcuts, Report Issue, About
```

### Context Menus
- **Editor:** AI Actions (`/explain`, `/refactor`, `/generate tests`), Refactor, Go To, Peek, Format
- **Explorer:** Git Stage/Unstage, Dispatch to Agent, New File/Folder, Reveal in Finder
- **Tabs:** Pin, Split Editor, Copy Path, Close Others, Agent Assign
- **Status Bar:** Branch switch, Sync, Model selector, Security tier, Notifications, Language/Encoding

### Command Palette Structure
- Hierarchical grouping: `File:`, `View:`, `Agent:`, `Git:`, `Terminal:`, `AI:/`
- Keybinding defaults: `Ctrl+Shift+P` (palette), `Ctrl+K Ctrl+S` (shortcuts), `Ctrl+Shift+M` (toggle Manager)
- Chord sequences: `Ctrl+K Ctrl+T` (theme), `Ctrl+K Ctrl+R` (reveal sidebar)

---

## 3. 🚀 ANTIGRAVITY SPECIAL FEATURES (EXTENDED)

### 3.1 🧠 The reasoning Engine (Agent Trace)
- **Visual Thought Process:** A collapsible panel that shows the LLM's step-by-step logic, tool calls, and failures.
- **Rewind/Edit Logic:** Ability for users to "fork" an agent's reasoning path if it's going the wrong way.

### 3.2 🖼️ Multimodal Requirement Input
- **Canvas Mode:** Drag/drop UI mocks or screenshots for Agent 1 (Planner) to extract CSS/React structures directly.
- **Voice-to-Logic:** Record architectural requirements; Antigravity converts voice to structured PRD artifacts.

### 3.3 🛡️ Zero-Trust Workspace
- **Sandbox Visualizer:** A real-time view of which network ports and directory paths the agent is currently accessing.
- **One-Click Revert:** Instant filesystem snapshot restoration if an autonomous agent edit fails.

### 3.4 🌐 Model Garden Mesh
- **Performance/Price Heatmap:** Shows which models are currently cheapest/fastest for specific sub-tasks (e.g., Qwen for regex, Claude for architecture).

---

## 4. ⚙️ OPTIONS & SETTINGS SCHEMA

| Setting Key | Type | Default | Scope | Description | Frontend Binding |
|-------------|------|---------|-------|-------------|------------------|
| `editor.autoSave` | `enum` | `"afterDelay"` | User/Workspace | Auto-save trigger behavior | `useAutoSave()` hook |
| `editor.formatOnSave` | `boolean` | `true` | User/Workspace | Run formatter on save | LSP `textDocument/formatting` |
| `editor.wordWrap` | `enum` | `"off"` | User/Workspace | Line wrapping strategy | Editor layout recomputation |
| `editor.minimap.enabled` | `boolean` | `true` | User/Workspace | Show minimap | Canvas/WebGPU minimap renderer |
| `agent.securityTier` | `enum` | `"auto"` | Workspace | Off/Auto/Turbo permission model | `SecurityContext` store |
| `agent.defaultModel` | `string` | `"claude-3.5-sonnet"` | User | Fallback model for agents | Model Garden selector state |
| `agent.stickyBrain` | `boolean` | `true` | User | Persist model per agent conversation | `AgentConfig` local storage |
| `ai.inlineCompletions` | `boolean` | `true` | User | Enable ghost text suggestions | Editor overlay renderer |
| `ai.artifactAutoPreview` | `boolean` | `true` | User | Auto-open artifact viewer on completion | `ArtifactPanel` visibility |
| `performance.gpuAcceleration` | `boolean` | `true` | User | Use WebGPU for editor rendering | Renderer pipeline init |

---

## 5. 🛠️ INTERACTION LOGIC (BUTTONS WORKING)

### 5.1 Event Handling Architecture
All UI interactions follow a **Signal-to-Command** pattern:
1. **Frontend (SolidJS/React):** User clicks a button (e.g., `Dispatch Agent`).
2. **Signal:** The UI sends an IPC (Inter-Process Communication) request to the Rust backend using `invoke('dispatch_task', { task_id })`.
3. **Backend Logic:** Rust validates the request, checks security tiers, and spawns the Agent process.
4. **State Sync:** Backend emits a stream of events (`agent_status_update`) which the frontend captures via `listen()` to update the `AgentCard` UI reactively.

### 5.2 Key Button Logics
- **`[Dispatch]`**: Validates current PRD artifact state → Checkpoints filesystem (Git) → Commits task to `AgentStore`.
- **`[Approve Artifact]`**: Moves file from `staging` (copy-on-write overlay) to `working_directory` → Marks task as `Completed` → Notifies next agent in DAG.
- **`[Security Tier Selector]`**: Updates the `SandboxPolicy` in Rust. If switching to `Turbo`, removes manual confirmation gates for terminal commands.
- **`[Switch View]`**: `Ctrl+Shift+M`. Performed via a layout transition engine that swaps the Editor buffer for the Manager DAG visualization without reloading the workspace.

---

## 6. 🎨 VISUAL DESIGN & LOOK/FEEL

### Layout Philosophy
- **Zero-chrome:** Minimal window decorations, content-first density
- **Floating panels:** Context-aware docks with blur/backdrop filters
- **Modal vs Modeless:** Modeless for AI chat/artifacts, modal for settings/security
- **Context-aware density:** Compact in manager surface, spacious in editor

### Status Bar Components
- Left: Cursor position, selection length, indentation, encoding
- Center: Active framework/LSP status, AI agent count, model latency
- Right: Git branch, sync status, security tier, notifications, GPU/render mode
- Interactions: Click to open dropdowns, drag to reorder, hover for tooltips

---

## 7. 🏗️ FRONTEND ARCHITECTURE & IMPLEMENTATION BLUEPRINT

### Framework & Stack Recommendation
- **UI Framework:** SolidJS (preferred for fine-grained reactivity)
- **Runtime:** Tauri v2 (Rust backend + Webview frontend)
- **Rendering:** WebGPU for editor gutter/minimap, DOM for UI panels
- **Terminal:** `xterm.js` + native PTY bridge via Rust
- **State:** Jotai/Zustand + RxJS for LSP/AI streams

### Component Hierarchy
```
AppShell (Layout)
├── TopMenu (Menu Bar)
├── ActivityBar (Icon Dock)
├── PrimaryPanel (Explorer/Search/SCM)
├── EditorSurface (Dual-Interface Router)
│   ├── EditorView (Monaco/Custom)
│   └── ManagerSurface (Mission Control)
│       ├── TeamDashboard
│       ├── WorkflowTimeline (DAG Visualization)
│       ├── AgentCard (x4)
│       └── ArtifactPreview
├── SecondaryPanel (Chat/Context/Terminal)
└── StatusBar (Bottom Bar)
```

---

## 8. 📐 DEFAULT PANEL LAYOUT DIAGRAM

```
┌─────────────────────────────────────────────────────────────────────┐
│ [TopMenu] File Edit View Agent Run Terminal Help                   │
├──────┬──────────────┬──────────────────────────────────────┬───────┤
│[Act  │[PrimaryPanel]│[EditorSurface]                      │[Second│
│ Bar] │ Explorer 📁  │ ┌────────────────────────────────┐ │ Panel]│
│ 📁🔍 │ Search 🔍    │ │ Editor View / Manager Surface  │ │ Chat 🤖│
│ 🌿▶️ │ SCM 🌿       │ │ (Tabbed, GPU-accelerated)      │ │ Art 📦│
│ 🧩 │ Run ▶️       │ └────────────────────────────────┘ │ Term 💻│
│ 🎯️ │ Extensions 🧩│                                      │       │
│      │ Agents 🤖    │                                      │       │
│      │ Manager 🎯   │                                      │       │
├──────┴──────────────┴──────────────────────────────────────┴───────┤
│ [StatusBar] Ln 12, Col 4 │ UTF-8 │ JavaScript │ 🌿 main │ 🤖 2 Active │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 🗺️ UI-TO-FRONTEND MAPPING TABLE (DYNAMICS)

| UI Element | Component Name | State Hook/Store | Interaction Logic |
|------------|----------------|------------------|-------------------|
| Agent Card | `<AgentCard />` | `useAgentStore()` | Clicking [Details] expands the Trace panel. |
| Workflow Timeline | `<WorkflowTimeline />` | `useWorkflowStore()` | Dragging nodes re-orders task dependencies. |
| Status Bar Tier | `<TierIndicator />` | `useSecurityStore()` | Click opens the security permission matrix. |
| Model Selector | `<ModelDropdown />` | `useModelStore()` | Changing model triggers a re-init of Agent context. |

---

## 📦 FRONTEND STATE SCHEMA OUTLINE

```typescript
interface AppState {
  layout: {
    activeView: 'editor' | 'manager';
    sidebar: { collapsed: boolean; activePanel: string; width: number };
  };
  agents: {
    teams: AgentTeam[];
    activeTeamId: string | null;
    agents: Record<string, AgentState>;
    reasoningTrace: Record<string, TraceStep[]>;
  };
  security: {
    tier: 'off' | 'auto' | 'turbo';
    pendingApprovals: ApprovalRequest[];
  };
  editor: {
    activeFile: string | null;
    gpu_mode: boolean;
  };
}
```

---

## ❓ ARCHITECTURAL DECISION (Clarification Response)

**Recommendation:** The blueprint prioritizes a **Hybrid Extension Model**. 
- **VS Code Extension Host:** Remains active for UI themes, LSP support, and language snippets to leverage the existing ecosystem.
- **Antigravity Native API:** A custom, low-latency Rust bridge (exposed via Tauri) specifically for Agent operations. This allows agents to register tools, intercept filesystem calls, and generate complex artifacts (like Mermaid/Browser renders) that traditional VS Code extensions cannot handle with high performance.

---

**Document Version:** 1.0  
**Status:** Unified Blueprint  
**Date:** April 2026
