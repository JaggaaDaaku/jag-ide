# Technical Requirements Document (TRD)
## **Jag IDE** - Agent-First Autonomous Development Platform

**Document Version:** 1.0  
**Date:** January 2025  
**Status:** Draft  
**Based on PRD Version:** 4.0

---

## TABLE OF CONTENTS

1. [Introduction](#1-introduction)
2. [System Architecture](#2-system-architecture)
3. [Technical Stack](#3-technical-stack)
4. [Multi-Agent System Architecture](#4-multi-agent-system-architecture)
5. [Component Specifications](#5-component-specifications)
6. [API Specifications](#6-api-specifications)
7. [Data Models & Database Schema](#7-data-models--database-schema)
8. [Model Integration Architecture](#8-model-integration-architecture)
9. [Security Architecture](#9-security-architecture)
10. [Performance Requirements](#10-performance-requirements)
11. [Implementation Plan](#11-implementation-plan)
12. [Testing Strategy](#12-testing-strategy)
13. [Deployment Architecture](#13-deployment-architecture)
14. [Integration Points](#14-integration-points)
15. [Technical Constraints & Assumptions](#15-technical-constraints--assumptions)
16. [Risk Assessment & Mitigation](#16-risk-assessment--mitigation)
17. [Appendices](#17-appendices)

---

## 1. INTRODUCTION

### 1.1 Purpose
This Technical Requirements Document (TRD) provides detailed technical specifications for implementing Jag IDE, translating the Product Requirements Document (PRD) into actionable engineering tasks, system architecture, and implementation guidelines.

### 1.2 Scope
- Multi-agent orchestration system with 4 specialized agents
- VS Code fork with Rust/C++ performance enhancements
- Embedded Ollama runtime with cloud model integration
- GPU-accelerated UI rendering
- Autonomous tool execution framework
- Artifact-based verification system

### 1.3 Definitions & Acronyms
| Term | Definition |
|------|------------|
| **A2A** | Agent-to-Agent Communication Protocol |
| **PRD** | Product Requirements Document |
| **TRD** | Technical Requirements Document |
| **LSP** | Language Server Protocol |
| **DAP** | Debug Adapter Protocol |
| **GPU** | Graphics Processing Unit |
| **pty** | Pseudo-terminal |
| **RAG** | Retrieval-Augmented Generation |
| **DAG** | Directed Acyclic Graph |
| **FFI** | Foreign Function Interface |

---

## 2. SYSTEM ARCHITECTURE

### 2.1 High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           PRESENTATION LAYER                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Electron/TypeScript UI                        │   │
│  │  - VS Code Fork (OSS)                                           │   │
│  │  - Mission Control Dashboard                                    │   │
│  │  - Editor View (Monaco Editor)                                  │   │
│  │  - GPU Rendering Layer (wgpu/GPUI patches)                      │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↕
┌─────────────────────────────────────────────────────────────────────────┐
│                        APPLICATION LAYER                                 │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │              Multi-Agent Orchestration Engine (Rust)             │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐ │   │
│  │  │  Agent 1    │ │  Agent 2    │ │  Agent 3    │ │  Agent 4  │ │   │
│  │  │  Planner    │ │  Backend    │ │  Frontend   │ │Integration│ │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘ │   │
│  │  - A2A Communication Bus (Tokio async)                         │   │
│  │  - Workflow Engine (DAG-based)                                 │   │
│  │  - Artifact Generator                                          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                  Extension Host (Node.js)                        │   │
│  │  - VS Code Extension API Compatibility                          │   │
│  │  - Plugin Sandboxing                                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↕
┌─────────────────────────────────────────────────────────────────────────┐
│                          SERVICE LAYER                                   │
│  ┌──────────────────────┐  ┌──────────────────────┐                    │
│  │  Model Router        │  │  Execution Sandbox   │                    │
│  │  (Python/Rust)       │  │  (Rust/C)            │                    │
│  │  - Ollama Client     │  │  - Terminal Runner   │                    │
│  │  - Cloud API Proxy   │  │  - File System       │                    │
│  │  - Context Manager   │  │  - Process Manager   │                    │
│  │  - Cost Optimizer    │  │  - Security Enforcer │                    │
│  └──────────────────────┘  └──────────────────────┘                    │
│  ┌──────────────────────┐  ┌──────────────────────┐                    │
│  │  LSP Server          │  │  Git Integration     │                    │
│  │  (Rust/C++)          │  │  (libgit2)           │                    │
│  │  - Language Support  │  │  - Diff Engine       │                    │
│  │  - IntelliSense      │  │  - Merge Conflict    │                    │
│  └──────────────────────┘  └──────────────────────┘                    │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↕
┌─────────────────────────────────────────────────────────────────────────┐
│                          DATA LAYER                                      │
│  ┌──────────────────────┐  ┌──────────────────────┐                    │
│  │  Workspace DB        │  │  Model Cache         │                    │
│  │  (SQLite/Redis)      │  │  (Local Storage)     │                    │
│  │  - Project Metadata  │  │  - GGUF Models       │                    │
│  │  - Agent State       │  │  - Embeddings        │                    │
│  │  - Artifact Store    │  │  - Vector Index      │                    │
│  └──────────────────────┘  └──────────────────────┘                    │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Architecture Principles

1. **Separation of Concerns:** Clear layering with well-defined interfaces
2. **Performance First:** Rust/C++ for compute-intensive tasks
3. **Security by Design:** Sandboxing, permission tiers, audit logging
4. **Extensibility:** Plugin architecture, VS Code API compatibility
5. **Modularity:** Microservices-style components with loose coupling
6. **Observability:** Comprehensive logging, metrics, tracing

### 2.3 Data Flow Architecture

```
User Input (Mission Control)
    ↓
[API Gateway - Rust Actix]
    ↓
[Agent Orchestrator]
    ├─→ Agent 1 (Planner) → Artifact Store
    ├─→ Agent 2 (Backend) → Code Generation
    ├─→ Agent 3 (Frontend) → UI Components
    └─→ Agent 4 (Integration) → Deployment
    ↓
[Execution Sandbox]
    ├─→ Terminal Commands
    ├─→ File System Operations
    └─→ Package Manager
    ↓
[Model Router]
    ├─→ Local Ollama (Gemma, Qwen)
    └─→ Cloud APIs (Claude, GPT)
    ↓
[Response Aggregation]
    ↓
[Artifact Generator]
    ↓
UI Rendering (GPU-accelerated)
```

---

## 3. TECHNICAL STACK

### 3.1 Core Technologies

| Component | Technology | Version | Rationale |
|-----------|------------|---------|-----------|
| **Base IDE** | VS Code OSS | 1.85+ | Extension ecosystem, familiar UI |
| **UI Framework** | Electron | 28.x | Cross-platform desktop |
| **Frontend Rendering** | React + TypeScript | 18.x / 5.x | Component-based UI |
| **Editor Component** | Monaco Editor | 0.45+ | VS Code's editor |
| **Agent Engine** | Rust | 1.75+ | Performance, safety, concurrency |
| **Async Runtime** | Tokio | 1.35+ | Async/await for agents |
| **Web Framework** | Actix-web | 4.4+ | High-performance HTTP server |
| **GPU Rendering** | wgpu | 0.19+ | Cross-platform GPU API |
| **Model Runtime** | Ollama | 0.1+ | Local LLM inference |
| **Python Bridge** | PyO3 | 0.20+ | Rust-Python FFI |
| **Database** | SQLite + Redis | 3.x / 7.x | Embedded + caching |
| **Git Integration** | libgit2 | 1.7+ | Native Git operations |
| **Terminal** | xterm.js + PTY | 5.x | Terminal emulation |

### 3.2 Build & Development Tools

```toml
# Cargo.toml (Rust dependencies)
[package]
name = "jag-ide-core"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
tokio-stream = "0.1"

# Web framework
actix-web = "4.4"
actix-cors = "0.7"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Ollama integration
reqwest = { version = "0.11", features = ["json"] }
ollama-rs = "0.1"

# Database
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
redis = "0.24"

# Git integration
git2 = "0.18"

# GPU rendering
wgpu = "0.19"
winit = "0.29"

# Python FFI
pyo3 = { version = "0.20", features = ["auto-initialize"] }

# Multi-agent coordination
dashmap = "5.5"
async-channel = "2.1"

# Cryptography
ring = "0.17"
jsonwebtoken = "9.2"

[dev-dependencies]
criterion = "0.5"
mockall = "0.12"
```

```json
// package.json (Electron/TypeScript)
{
  "name": "jag-ide",
  "version": "0.1.0",
  "main": "dist/main.js",
  "scripts": {
    "start": "electron .",
    "build": "tsc && webpack",
    "test": "jest"
  },
  "dependencies": {
    "electron": "^28.0.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "monaco-editor": "^0.45.0",
    "@vscode/codicons": "^0.0.35",
    "xterm": "^5.3.0",
    "xterm-addon-fit": "^0.8.0"
  },
  "devDependencies": {
    "typescript": "^5.3.0",
    "webpack": "^5.89.0",
    "jest": "^29.7.0",
    "electron-builder": "^24.9.0"
  }
}
```

### 3.3 Platform Support Matrix

| Platform | OS Version | Architecture | Build Target |
|----------|------------|--------------|--------------|
| **Windows** | 10/11 (64-bit) | x86_64 | `x86_64-pc-windows-msvc` |
| **Windows** | 10/11 (ARM) | ARM64 | `aarch64-pc-windows-msvc` |
| **macOS** | 11+ (Big Sur) | Intel | `x86_64-apple-darwin` |
| **macOS** | 11+ (Big Sur) | Apple Silicon | `aarch64-apple-darwin` |
| **Linux** | Ubuntu 20.04+ | x86_64 | `x86_64-unknown-linux-gnu` |
| **Linux** | Ubuntu 20.04+ | ARM64 | `aarch64-unknown-linux-gnu` |

---

## 4. MULTI-AGENT SYSTEM ARCHITECTURE

### 4.1 Agent Orchestration Engine

```rust
// src/agents/orchestrator.rs
use tokio::sync::broadcast;
use dashmap::DashMap;
use uuid::Uuid;

pub struct AgentOrchestrator {
    agents: DashMap<AgentId, AgentInstance>,
    workflow_engine: WorkflowEngine,
    artifact_store: ArtifactStore,
    message_bus: broadcast::Sender<AgentMessage>,
    security_policy: SecurityPolicy,
}

pub struct AgentInstance {
    pub id: AgentId,
    pub role: AgentRole,
    pub state: AgentState,
    pub context: AgentContext,
    pub model_router: ModelRouter,
    pub tools: ToolRegistry,
    pub task_queue: Vec<Task>,
}

impl AgentOrchestrator {
    pub async fn spawn_agent_team(&self, project_id: ProjectId) -> Result<TeamId> {
        let team_id = TeamId::new(Uuid::new_v4());
        
        // Create 4 specialized agents
        let agent_1 = self.create_agent(AgentRole::Planner, team_id).await?;
        let agent_2 = self.create_agent(AgentRole::Backend, team_id).await?;
        let agent_3 = self.create_agent(AgentRole::Frontend, team_id).await?;
        let agent_4 = self.create_agent(AgentRole::Integration, team_id).await?;
        
        // Set up workflow dependencies
        self.workflow_engine.setup_dependencies(&[
            (agent_1.id, vec![]),
            (agent_2.id, vec![agent_1.id]),
            (agent_3.id, vec![agent_2.id]),
            (agent_4.id, vec![agent_3.id]),
        ]).await?;
        
        Ok(team_id)
    }
    
    pub async fn dispatch_task(&self, agent_id: AgentId, task: Task) -> Result<()> {
        let agent = self.agents.get_mut(&agent_id)
            .ok_or(Error::AgentNotFound)?;
        
        // Check security policy
        self.security_policy.validate_task(&task, &agent.state)?;
        
        // Add to task queue
        agent.task_queue.push(task);
        
        // Notify agent
        self.message_bus.send(AgentMessage::TaskAvailable(agent_id))?;
        
        Ok(())
    }
}
```

### 4.2 Agent-to-Agent (A2A) Communication Protocol

```rust
// src/agents/communication.rs
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: MessageId,
    pub from: AgentId,
    pub to: AgentId,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub payload: MessagePayload,
    pub priority: Priority,
    pub correlation_id: Option<MessageId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    ArtifactReady(ArtifactType),
    TaskRequest(TaskType),
    StatusUpdate(AgentStatus),
    DependencyResolved(TaskId),
    ErrorReport(ErrorDetails),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub artifact_id: Option<ArtifactId>,
    pub task_id: Option<TaskId>,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

// A2A Message Bus Implementation
pub struct A2AMessageBus {
    tx: broadcast::Sender<AgentMessage>,
    subscribers: DashMap<AgentId, broadcast::Receiver<AgentMessage>>,
}

impl A2AMessageBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            tx,
            subscribers: DashMap::new(),
        }
    }
    
    pub fn subscribe(&self, agent_id: AgentId) -> broadcast::Receiver<AgentMessage> {
        let rx = self.tx.subscribe();
        self.subscribers.insert(agent_id, rx.resubscribe());
        rx
    }
    
    pub async fn send(&self, message: AgentMessage) -> Result<()> {
        self.tx.send(message)?;
        Ok(())
    }
    
    pub async fn broadcast(&self, message: AgentMessage) -> Result<()> {
        self.tx.send(message)?;
        Ok(())
    }
}
```

### 4.3 Workflow Engine (DAG-based)

```rust
// src/workflow/engine.rs
use petgraph::graph::DiGraph;
use petgraph::algo::toposort;

pub struct WorkflowEngine {
    workflow_graph: DiGraph<TaskNode, DependencyType>,
    task_registry: HashMap<TaskId, Task>,
    execution_state: ExecutionState,
}

pub struct TaskNode {
    pub id: TaskId,
    pub agent_id: AgentId,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub dependencies: Vec<TaskId>,
    pub artifacts: Vec<ArtifactId>,
    pub retry_count: u32,
    pub max_retries: u32,
}

pub enum DependencyType {
    Hard,      // Must complete before
    Soft,      // Should complete before
    Parallel,  // Can run in parallel
}

impl WorkflowEngine {
    pub async fn execute_workflow(&mut self, workflow_id: WorkflowId) -> Result<()> {
        // Topological sort for execution order
        let sorted_tasks = toposort(&self.workflow_graph, None)
            .map_err(|cycle| Error::CircularDependency)?;
        
        // Execute tasks in order
        for task_node in sorted_tasks {
            let task = self.task_registry.get_mut(&task_node.id)
                .ok_or(Error::TaskNotFound)?;
            
            // Check dependencies
            if !self.check_dependencies(task).await? {
                continue;
            }
            
            // Execute task
            match self.execute_task(task).await {
                Ok(_) => task.status = TaskStatus::Completed,
                Err(e) => {
                    if task.retry_count < task.max_retries {
                        task.retry_count += 1;
                        // Retry logic
                    } else {
                        task.status = TaskStatus::Failed;
                        return Err(e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn check_dependencies(&self, task: &Task) -> Result<bool> {
        for dep_id in &task.dependencies {
            let dep_task = self.task_registry.get(dep_id)
                .ok_or(Error::DependencyNotFound)?;
            
            if dep_task.status != TaskStatus::Completed {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
```

### 4.4 Agent Specialization Implementations

#### Agent 1: Product Architect & System Planner

```rust
// src/agents/planner.rs
pub struct PlannerAgent {
    id: AgentId,
    model: ModelRouter,
    workspace: WorkspaceContext,
}

impl Agent for PlannerAgent {
    async fn execute(&self, task: Task) -> Result<Artifact> {
        match task.task_type {
            TaskType::GeneratePRD => self.generate_prd(task).await,
            TaskType::DesignArchitecture => self.design_architecture(task).await,
            TaskType::DefineDataModels => self.define_data_models(task).await,
            TaskType::SpecifyAPIs => self.specify_apis(task).await,
            _ => Err(Error::UnsupportedTaskType),
        }
    }
}

impl PlannerAgent {
    async fn generate_prd(&self, task: Task) -> Result<Artifact> {
        let prompt = format!(
            r#"Analyze the following project requirements and generate a comprehensive PRD:
            
            Project: {}
            Description: {}
            Tech Stack Preferences: {:?}
            
            Generate:
            1. Core features list
            2. User stories
            3. Functional requirements
            4. Non-functional requirements
            5. Success metrics
            "#,
            task.project_name,
            task.description,
            task.tech_preferences
        );
        
        let response = self.model.generate(&prompt, ModelPreference::Reasoning).await?;
        
        let prd: ProductRequirementsDocument = serde_json::from_str(&response)?;
        
        Ok(Artifact {
            id: ArtifactId::new(),
            artifact_type: ArtifactType::PRD,
            content: serde_json::to_vec(&prd)?,
            metadata: ArtifactMetadata {
                created_by: self.id,
                timestamp: Utc::now(),
                version: "1.0",
            },
        })
    }
    
    async fn design_architecture(&self, task: Task) -> Result<Artifact> {
        // Generate system architecture diagram using Mermaid
        let architecture = SystemArchitecture {
            components: self.identify_components(&task).await?,
            relationships: self.define_relationships().await?,
            data_flow: self.map_data_flow().await?,
        };
        
        let mermaid_diagram = self.render_mermaid(&architecture).await?;
        
        Ok(Artifact {
            id: ArtifactId::new(),
            artifact_type: ArtifactType::ArchitectureDiagram,
            content: mermaid_diagram.into_bytes(),
            metadata: ArtifactMetadata::new(self.id),
        })
    }
}
```

#### Agent 2: Backend Engineer

```rust
// src/agents/backend.rs
pub struct BackendAgent {
    id: AgentId,
    model: ModelRouter,
    workspace: WorkspaceContext,
    code_generator: CodeGenerator,
}

impl BackendAgent {
    async fn implement_apis(&self, task: Task) -> Result<Artifact> {
        let api_spec = self.load_api_specification(task.api_spec_id).await?;
        
        let mut generated_code = HashMap::new();
        
        for endpoint in &api_spec.endpoints {
            let code = self.generate_endpoint_code(endpoint).await?;
            generated_code.insert(endpoint.path.clone(), code);
        }
        
        // Generate database models
        let models = self.generate_models(&api_spec.data_models).await?;
        
        // Generate migrations
        let migrations = self.generate_migrations(&api_spec.data_models).await?;
        
        Ok(Artifact {
            id: ArtifactId::new(),
            artifact_type: ArtifactType::BackendCode,
            content: serde_json::to_vec(&BackendArtifact {
                apis: generated_code,
                models,
                migrations,
                config: self.generate_config().await?,
            })?,
            metadata: ArtifactMetadata::new(self.id),
        })
    }
    
    async fn generate_endpoint_code(&self, endpoint: &Endpoint) -> Result<String> {
        let prompt = format!(
            r#"Generate a {} endpoint for {} with the following specification:
            
            Method: {}
            Path: {}
            Request Body: {:?}
            Response: {:?}
            Authentication: {:?}
            
            Use Express.js (Node.js) with TypeScript.
            Include:
            - Route handler
            - Input validation
            - Error handling
            - Database operations
            "#,
            endpoint.method,
            endpoint.description,
            endpoint.method,
            endpoint.path,
            endpoint.request_schema,
            endpoint.response_schema,
            endpoint.auth_required
        );
        
        self.model.generate(&prompt, ModelPreference::CodeGeneration).await
    }
}
```

#### Agent 3: Frontend Developer

```rust
// src/agents/frontend.rs
pub struct FrontendAgent {
    id: AgentId,
    model: ModelRouter,
    workspace: WorkspaceContext,
    ui_generator: UIGenerator,
}

impl FrontendAgent {
    async fn build_ui_components(&self, task: Task) -> Result<Artifact> {
        let pages = self.identify_required_pages(&task).await?;
        let mut components = HashMap::new();
        
        for page in pages {
            let page_component = self.generate_page_component(&page).await?;
            components.insert(page.name.clone(), page_component);
            
            // Generate child components
            for child in page.child_components {
                let component = self.generate_reusable_component(&child).await?;
                components.insert(child.name.clone(), component);
            }
        }
        
        // Generate state management
        let state_management = self.generate_state_management(&task).await?;
        
        // Generate API integration layer
        let api_client = self.generate_api_client(&task.api_spec).await?;
        
        Ok(Artifact {
            id: ArtifactId::new(),
            artifact_type: ArtifactType::FrontendCode,
            content: serde_json::to_vec(&FrontendArtifact {
                components,
                state_management,
                api_client,
                styles: self.generate_styles().await?,
            })?,
            metadata: ArtifactMetadata::new(self.id),
        })
    }
    
    async fn generate_page_component(&self, page: &PageSpec) -> Result<Component> {
        let prompt = format!(
            r#"Create a React component for the {} page with the following requirements:
            
            Route: {}
            Features: {:?}
            API Endpoints: {:?}
            State Requirements: {:?}
            
            Use:
            - React functional components with hooks
            - TypeScript for type safety
            - Tailwind CSS for styling
            - React Query for data fetching
            "#,
            page.name,
            page.route,
            page.features,
            page.api_endpoints,
            page.state_requirements
        );
        
        let code = self.model.generate(&prompt, ModelPreference::CodeGeneration).await?;
        
        Ok(Component {
            name: page.name.clone(),
            code,
            file_path: format!("src/pages/{}.tsx", page.name.to_lowercase()),
            dependencies: self.extract_dependencies(&code).await?,
        })
    }
}
```

#### Agent 4: Integration & DevOps Specialist

```rust
// src/agents/integration.rs
pub struct IntegrationAgent {
    id: AgentId,
    model: ModelRouter,
    workspace: WorkspaceContext,
    test_runner: TestRunner,
    deployment_manager: DeploymentManager,
}

impl IntegrationAgent {
    async fn integrate_and_deploy(&self, task: Task) -> Result<Artifact> {
        // Load artifacts from previous agents
        let backend_code = self.load_artifact(task.backend_artifact_id).await?;
        let frontend_code = self.load_artifact(task.frontend_artifact_id).await?;
        
        // Set up project structure
        let project_structure = self.setup_project_structure(&task).await?;
        
        // Integrate frontend and backend
        self.integrate_frontend_backend(&backend_code, &frontend_code).await?;
        
        // Configure environment
        self.setup_environment_variables(&task).await?;
        
        // Run tests
        let test_results = self.run_integration_tests().await?;
        
        if !test_results.all_passed() {
            return Err(Error::TestFailed(test_results.failures));
        }
        
        // Set up local development server
        let localhost_url = self.start_development_server().await?;
        
        // Generate deployment configuration
        let docker_config = self.generate_docker_config().await?;
        let ci_cd_config = self.generate_ci_cd_config().await?;
        
        Ok(Artifact {
            id: ArtifactId::new(),
            artifact_type: ArtifactType::DeploymentPackage,
            content: serde_json::to_vec(&DeploymentArtifact {
                project_structure,
                test_results,
                localhost_url,
                docker_config,
                ci_cd_config,
                setup_instructions: self.generate_setup_instructions().await?,
            })?,
            metadata: ArtifactMetadata::new(self.id),
        })
    }
    
    async fn run_integration_tests(&self) -> Result<TestResults> {
        // Run E2E tests using Playwright
        let e2e_tests = self.run_playwright_tests().await?;
        
        // Run API tests
        let api_tests = self.run_api_tests().await?;
        
        // Run performance tests
        let perf_tests = self.run_performance_tests().await?;
        
        Ok(TestResults {
            e2e: e2e_tests,
            api: api_tests,
            performance: perf_tests,
            all_passed: e2e_tests.passed && api_tests.passed && perf_tests.passed,
        })
    }
}
```

---

## 5. COMPONENT SPECIFICATIONS

### 5.1 Model Router

```rust
// src/model/router.rs
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ModelRouter {
    local_models: Arc<RwLock<Vec<LocalModel>>>,
    cloud_providers: Arc<RwLock<Vec<CloudProvider>>>,
    context_manager: ContextManager,
    cost_tracker: CostTracker,
    fallback_strategy: FallbackStrategy,
}

pub struct LocalModel {
    pub id: ModelId,
    pub name: String,
    pub provider: ModelProvider,
    pub capabilities: Vec<Capability>,
    pub context_window: usize,
    pub quantization: QuantizationLevel,
    pub status: ModelStatus,
}

pub struct CloudProvider {
    pub id: ProviderId,
    pub name: String,
    pub api_endpoint: String,
    pub api_key: String,
    pub models: Vec<CloudModel>,
    pub rate_limit: RateLimit,
    pub cost_per_token: CostConfig,
}

impl ModelRouter {
    pub async fn generate(&self, prompt: &str, preference: ModelPreference) -> Result<String> {
        let model = self.select_model(preference).await?;
        
        let request = GenerationRequest {
            model: model.id.clone(),
            prompt: prompt.to_string(),
            max_tokens: self.calculate_max_tokens(prompt, &model).await?,
            temperature: self.calculate_temperature(preference),
            stream: false,
        };
        
        let response = match model.provider {
            ModelProvider::Local => self.generate_local(request).await?,
            ModelProvider::Cloud(cloud_id) => self.generate_cloud(request, cloud_id).await?,
        };
        
        // Track cost
        self.cost_tracker.track_usage(&model, &response.usage).await?;
        
        Ok(response.text)
    }
    
    pub async fn select_model(&self, preference: ModelPreference) -> Result<Model> {
        match preference {
            ModelPreference::Reasoning => {
                // Prefer Claude/GPT for complex reasoning
                self.cloud_providers.read().await
                    .iter()
                    .find(|p| p.name == "Anthropic" || p.name == "OpenAI")
                    .and_then(|p| p.models.first())
                    .map(|m| Model::Cloud(m.clone()))
                    .ok_or(Error::NoModelAvailable)
            }
            ModelPreference::CodeGeneration => {
                // Prefer Qwen/CodeLlama for code
                self.local_models.read().await
                    .iter()
                    .find(|m| m.capabilities.contains(&Capability::CodeGeneration))
                    .map(|m| Model::Local(m.clone()))
                    .or_else(|| {
                        self.cloud_providers.read().await
                            .iter()
                            .find(|p| p.models.iter().any(|m| m.capabilities.contains(&Capability::CodeGeneration)))
                            .and_then(|p| p.models.first())
                            .map(|m| Model::Cloud(m.clone()))
                    })
                    .ok_or(Error::NoModelAvailable)
            }
            ModelPreference::Fast => {
                // Prefer local models for speed
                self.local_models.read().await
                    .iter()
                    .find(|m| m.status == ModelStatus::Ready)
                    .map(|m| Model::Local(m.clone()))
                    .ok_or(Error::NoModelAvailable)
            }
        }
    }
    
    async fn generate_local(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        // Use Ollama API
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:11434/api/generate")
            .json(&serde_json::json!({
                "model": request.model,
                "prompt": request.prompt,
                "stream": request.stream,
            }))
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;
        
        Ok(GenerationResponse {
            text: response.response,
            usage: TokenUsage {
                prompt_tokens: response.prompt_eval_count,
                completion_tokens: response.eval_count,
                total_tokens: response.prompt_eval_count + response.eval_count,
            },
        })
    }
}
```

### 5.2 Execution Sandbox

```rust
// src/sandbox/executor.rs
use std::process::{Command, Stdio};
use tokio::process::Child;
use nix::unistd::{Uid, Gid};

pub struct ExecutionSandbox {
    workspace_root: PathBuf,
    security_policy: SecurityPolicy,
    resource_limits: ResourceLimits,
    audit_logger: AuditLogger,
}

pub struct SecurityPolicy {
    pub tier: SecurityTier,
    pub allowed_commands: Vec<String>,
    pub denied_commands: Vec<String>,
    pub allowed_paths: Vec<PathBuf>,
    pub max_file_size: u64,
    pub max_memory_mb: u64,
    pub network_access: bool,
}

pub enum SecurityTier {
    Off,      // All actions require approval
    Auto,     // Safe operations auto-approved
    Turbo,    // Full autonomy
}

impl ExecutionSandbox {
    pub async fn execute_command(&self, command: CommandSpec) -> Result<CommandOutput> {
        // Validate against security policy
        self.validate_command(&command).await?;
        
        // Log for audit
        self.audit_logger.log_command(&command).await?;
        
        // Execute with resource limits
        let output = self.execute_with_limits(command).await?;
        
        // Validate output
        self.validate_output(&output).await?;
        
        Ok(output)
    }
    
    async fn validate_command(&self, command: &CommandSpec) -> Result<()> {
        match self.security_policy.tier {
            SecurityTier::Off => {
                // Always require approval
                Err(Error::ApprovalRequired)
            }
            SecurityTier::Auto => {
                // Check if command is safe
                if self.is_safe_command(command) {
                    Ok(())
                } else {
                    Err(Error::ApprovalRequired)
                }
            }
            SecurityTier::Turbo => {
                // Allow all commands within policy
                if self.is_denied_command(command) {
                    Err(Error::CommandDenied)
                } else {
                    Ok(())
                }
            }
        }
    }
    
    fn is_safe_command(&self, command: &CommandSpec) -> bool {
        let safe_commands = vec!["ls", "cat", "grep", "find", "echo", "pwd", "git status"];
        safe_commands.contains(&command.program.as_str())
    }
    
    fn is_denied_command(&self, command: &CommandSpec) -> bool {
        let denied_commands = vec!["rm -rf /", "mkfs", "dd", "chmod 777"];
        denied_commands.iter().any(|d| command.to_string().contains(d))
    }
    
    async fn execute_with_limits(&self, command: CommandSpec) -> Result<CommandOutput> {
        let mut cmd = Command::new(&command.program);
        cmd.args(&command.args);
        cmd.current_dir(&self.workspace_root);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        // Set resource limits (Linux-specific)
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::process::CommandExt;
            
            let limits = self.resource_limits.clone();
            unsafe {
                cmd.pre_exec(move || {
                    // Set memory limit
                    let rlim = libc::rlimit {
                        rlim_cur: limits.max_memory_mb * 1024 * 1024,
                        rlim_max: limits.max_memory_mb * 1024 * 1024,
                    };
                    libc::setrlimit(libc::RLIMIT_AS, &rlim);
                    
                    // Set CPU time limit
                    let rlim = libc::rlimit {
                        rlim_cur: limits.max_cpu_seconds,
                        rlim_max: limits.max_cpu_seconds,
                    };
                    libc::setrlimit(libc::RLIMIT_CPU, &rlim);
                    
                    Ok(())
                });
            }
        }
        
        // Execute with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.resource_limits.timeout_seconds),
            cmd.output()
        ).await??;
        
        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            execution_time: output.metadata.map(|m| m.elapsed()).unwrap_or_default(),
        })
    }
    
    pub async fn modify_file(&self, file_path: &Path, content: &[u8], operation: FileOperation) -> Result<()> {
        // Validate path
        self.validate_file_path(file_path).await?;
        
        // Create backup
        self.create_backup(file_path).await?;
        
        // Execute operation
        match operation {
            FileOperation::Create => {
                tokio::fs::write(file_path, content).await?;
            }
            FileOperation::Update => {
                tokio::fs::write(file_path, content).await?;
            }
            FileOperation::Delete => {
                tokio::fs::remove_file(file_path).await?;
            }
        }
        
        // Log for audit
        self.audit_logger.log_file_operation(file_path, &operation).await?;
        
        Ok(())
    }
}
```

### 5.3 Artifact Generator

```rust
// src/artifacts/generator.rs
use serde::{Serialize, Deserialize};
use tera::Tera;

pub struct ArtifactGenerator {
    template_engine: Tera,
    mermaid_renderer: MermaidRenderer,
    diff_engine: DiffEngine,
}

pub enum ArtifactType {
    PRD,
    ArchitectureDiagram,
    APISpecification,
    DatabaseSchema,
    BackendCode,
    FrontendCode,
    TestReport,
    DeploymentPackage,
}

pub struct Artifact {
    pub id: ArtifactId,
    pub artifact_type: ArtifactType,
    pub content: Vec<u8>,
    pub metadata: ArtifactMetadata,
    pub verification_status: VerificationStatus,
}

impl ArtifactGenerator {
    pub async fn generate_prd_artifact(&self, data: &PRDData) -> Result<Artifact> {
        let template = self.template_engine.get_template("prd.md")?;
        
        let rendered = template.render(&tera::Context::from_serialize(data)?)?;
        
        Ok(Artifact {
            id: ArtifactId::new(),
            artifact_type: ArtifactType::PRD,
            content: rendered.into_bytes(),
            metadata: ArtifactMetadata {
                created_at: Utc::now(),
                format: "markdown",
                size: rendered.len(),
            },
            verification_status: VerificationStatus::Pending,
        })
    }
    
    pub async fn generate_architecture_diagram(&self, data: &ArchitectureData) -> Result<Artifact> {
        let mermaid_code = self.mermaid_renderer.render_system_architecture(data).await?;
        
        // Render to SVG
        let svg = self.mermaid_renderer.render_to_svg(&mermaid_code).await?;
        
        Ok(Artifact {
            id: ArtifactId::new(),
            artifact_type: ArtifactType::ArchitectureDiagram,
            content: svg.into_bytes(),
            metadata: ArtifactMetadata {
                created_at: Utc::now(),
                format: "svg",
                size: svg.len(),
            },
            verification_status: VerificationStatus::Pending,
        })
    }
    
    pub async fn generate_code_diff(&self, old_code: &str, new_code: &str) -> Result<Artifact> {
        let diff = self.diff_engine.generate_diff(old_code, new_code).await?;
        
        Ok(Artifact {
            id: ArtifactId::new(),
            artifact_type: ArtifactType::CodeDiff,
            content: diff.into_bytes(),
            metadata: ArtifactMetadata {
                created_at: Utc::now(),
                format: "unified-diff",
                size: diff.len(),
            },
            verification_status: VerificationStatus::Pending,
        })
    }
    
    pub async fn generate_test_report(&self, test_results: &TestResults) -> Result<Artifact> {
        let template = self.template_engine.get_template("test_report.html")?;
        
        let rendered = template.render(&tera::Context::from_serialize(test_results)?)?;
        
        Ok(Artifact {
            id: ArtifactId::new(),
            artifact_type: ArtifactType::TestReport,
            content: rendered.into_bytes(),
            metadata: ArtifactMetadata {
                created_at: Utc::now(),
                format: "html",
                size: rendered.len(),
            },
            verification_status: VerificationStatus::Pending,
        })
    }
}
```

### 5.4 Workspace Manager

```rust
// src/workspace/manager.rs
use std::path::PathBuf;
use sqlx::SqlitePool;

pub struct WorkspaceManager {
    db_pool: SqlitePool,
    workspace_root: PathBuf,
    file_watcher: FileWatcher,
}

pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub root_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub agents: Vec<AgentId>,
    pub artifacts: Vec<ArtifactId>,
}

impl WorkspaceManager {
    pub async fn create_workspace(&self, name: &str) -> Result<Workspace> {
        let workspace_id = WorkspaceId::new();
        let root_path = self.workspace_root.join(workspace_id.to_string());
        
        // Create directory structure
        tokio::fs::create_dir_all(&root_path).await?;
        
        // Initialize .jag directory for metadata
        let jag_dir = root_path.join(".jag");
        tokio::fs::create_dir_all(&jag_dir).await?;
        
        // Create workspace record in database
        let workspace = Workspace {
            id: workspace_id,
            name: name.to_string(),
            root_path: root_path.clone(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            agents: vec![],
            artifacts: vec![],
        };
        
        sqlx::query!(
            r#"
            INSERT INTO workspaces (id, name, root_path, created_at, modified_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            workspace.id,
            workspace.name,
            workspace.root_path.display().to_string(),
            workspace.created_at,
            workspace.modified_at,
        )
        .execute(&self.db_pool)
        .await?;
        
        // Start file watcher
        self.file_watcher.watch(&root_path).await?;
        
        Ok(workspace)
    }
    
    pub async fn get_file(&self, workspace_id: WorkspaceId, file_path: &Path) -> Result<FileContent> {
        let workspace = self.get_workspace(workspace_id).await?;
        let full_path = workspace.root_path.join(file_path);
        
        // Validate path is within workspace
        if !full_path.starts_with(&workspace.root_path) {
            return Err(Error::PathTraversal);
        }
        
        let content = tokio::fs::read_to_string(&full_path).await?;
        
        Ok(FileContent {
            path: file_path.to_path_buf(),
            content,
            size: content.len(),
            last_modified: self.get_file_metadata(&full_path).await?.modified,
        })
    }
    
    pub async fn save_file(&self, workspace_id: WorkspaceId, file_path: &Path, content: &str) -> Result<()> {
        let workspace = self.get_workspace(workspace_id).await?;
        let full_path = workspace.root_path.join(file_path);
        
        // Validate path
        if !full_path.starts_with(&workspace.root_path) {
            return Err(Error::PathTraversal);
        }
        
        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Write file
        tokio::fs::write(&full_path, content).await?;
        
        // Update workspace modified time
        sqlx::query!(
            "UPDATE workspaces SET modified_at = ? WHERE id = ?",
            Utc::now(),
            workspace_id,
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
}
```

---

## 6. API SPECIFICATIONS

### 6.1 REST API Endpoints

#### Agent Management API

```yaml
openapi: 3.0.0
info:
  title: Jag IDE Agent API
  version: 1.0.0

paths:
  /api/v1/agents:
    post:
      summary: Create a new agent team
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateAgentTeamRequest'
      responses:
        '201':
          description: Agent team created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AgentTeamResponse'
    
    get:
      summary: List all active agents
      responses:
        '200':
          description: List of agents
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/AgentStatus'

  /api/v1/agents/{agentId}/tasks:
    post:
      summary: Dispatch task to agent
      parameters:
        - name: agentId
          in: path
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/TaskRequest'
      responses:
        '202':
          description: Task accepted
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TaskResponse'

  /api/v1/agents/{agentId}/artifacts:
    get:
      summary: Get artifacts generated by agent
      parameters:
        - name: agentId
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: List of artifacts
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Artifact'

  /api/v1/workspaces/{workspaceId}/artifacts/{artifactId}/approve:
    post:
      summary: Approve artifact
      parameters:
        - name: workspaceId
          in: path
          required: true
          schema:
            type: string
        - name: artifactId
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Artifact approved
        '400':
          description: Invalid state transition

components:
  schemas:
    CreateAgentTeamRequest:
      type: object
      required:
        - workspaceId
        - projectDescription
      properties:
        workspaceId:
          type: string
        projectDescription:
          type: string
        securityTier:
          type: string
          enum: [off, auto, turbo]
        modelPreferences:
          type: object
          additionalProperties:
            type: string

    AgentTeamResponse:
      type: object
      properties:
        teamId:
          type: string
        agents:
          type: array
          items:
            $ref: '#/components/schemas/Agent'
        status:
          type: string

    TaskRequest:
      type: object
      required:
        - taskType
        - payload
      properties:
        taskType:
          type: string
          enum: [generate_prd, implement_api, build_ui, integrate]
        payload:
          type: object
        priority:
          type: string
          enum: [low, normal, high, critical]

    Artifact:
      type: object
      properties:
        id:
          type: string
        type:
          type: string
        createdAt:
          type: string
          format: date-time
        status:
          type: string
          enum: [pending, approved, rejected]
        downloadUrl:
          type: string
```

### 6.2 WebSocket API for Real-time Updates

```typescript
// WebSocket message types
interface WSMessage {
  type: MessageType;
  payload: any;
  timestamp: string;
}

enum MessageType {
  AGENT_STATUS_UPDATE = 'agent:status',
  TASK_PROGRESS = 'task:progress',
  ARTIFACT_READY = 'artifact:ready',
  ERROR = 'error',
  LOG = 'log',
}

interface AgentStatusUpdate {
  agentId: string;
  status: 'idle' | 'working' | 'completed' | 'error';
  currentTask?: string;
  progress?: number;
}

interface TaskProgress {
  taskId: string;
  agentId: string;
  progress: number; // 0-100
  message: string;
  artifacts?: Artifact[];
}

// Client-side WebSocket connection
class JagIDEWebSocket {
  private ws: WebSocket;
  private messageHandlers: Map<MessageType, (payload: any) => void>;

  constructor(url: string) {
    this.ws = new WebSocket(url);
    this.messageHandlers = new Map();
    this.setupEventListeners();
  }

  private setupEventListeners() {
    this.ws.onmessage = (event) => {
      const message: WSMessage = JSON.parse(event.data);
      const handler = this.messageHandlers.get(message.type);
      if (handler) {
        handler(message.payload);
      }
    };
  }

  onAgentStatusUpdate(handler: (status: AgentStatusUpdate) => void) {
    this.messageHandlers.set(MessageType.AGENT_STATUS_UPDATE, handler);
  }

  onTaskProgress(handler: (progress: TaskProgress) => void) {
    this.messageHandlers.set(MessageType.TASK_PROGRESS, handler);
  }

  onArtifactReady(handler: (artifact: Artifact) => void) {
    this.messageHandlers.set(MessageType.ARTIFACT_READY, handler);
  }

  subscribeToAgent(agentId: string) {
    this.send({
      type: 'subscribe:agent',
      payload: { agentId },
    });
  }
}
```

---

## 7. DATA MODELS & DATABASE SCHEMA

### 7.1 Database Schema (SQLite)

```sql
-- Workspaces table
CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    settings_json TEXT
);

CREATE INDEX idx_workspaces_name ON workspaces(name);

-- Agents table
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('planner', 'backend', 'frontend', 'integration')),
    status TEXT NOT NULL DEFAULT 'idle' CHECK (status IN ('idle', 'working', 'completed', 'error')),
    model_id TEXT,
    security_tier TEXT NOT NULL DEFAULT 'auto',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_active_at DATETIME,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE INDEX idx_agents_workspace ON agents(workspace_id);
CREATE INDEX idx_agents_status ON agents(status);

-- Tasks table
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    task_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    priority INTEGER NOT NULL DEFAULT 0,
    payload_json TEXT NOT NULL,
    result_json TEXT,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    started_at DATETIME,
    completed_at DATETIME,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

CREATE INDEX idx_tasks_agent ON tasks(agent_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_created ON tasks(created_at);

-- Task dependencies (DAG)
CREATE TABLE task_dependencies (
    task_id TEXT NOT NULL,
    depends_on_task_id TEXT NOT NULL,
    dependency_type TEXT NOT NULL DEFAULT 'hard' CHECK (dependency_type IN ('hard', 'soft', 'parallel')),
    PRIMARY KEY (task_id, depends_on_task_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (depends_on_task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- Artifacts table
CREATE TABLE artifacts (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    task_id TEXT,
    artifact_type TEXT NOT NULL,
    content_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    format TEXT NOT NULL,
    verification_status TEXT NOT NULL DEFAULT 'pending' CHECK (verification_status IN ('pending', 'approved', 'rejected')),
    metadata_json TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    verified_at DATETIME,
    verified_by TEXT,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL
);

CREATE INDEX idx_artifacts_workspace ON artifacts(workspace_id);
CREATE INDEX idx_artifacts_agent ON artifacts(agent_id);
CREATE INDEX idx_artifacts_type ON artifacts(artifact_type);
CREATE INDEX idx_artifacts_status ON artifacts(verification_status);

-- Models table
CREATE TABLE models (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider TEXT NOT NULL CHECK (provider IN ('ollama', 'anthropic', 'openai', 'google', 'mistral')),
    model_type TEXT NOT NULL CHECK (model_type IN ('local', 'cloud')),
    capabilities_json TEXT,
    context_window INTEGER,
    quantization_level TEXT,
    status TEXT NOT NULL DEFAULT 'available' CHECK (status IN ('available', 'downloading', 'ready', 'error')),
    cost_per_1k_tokens REAL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME
);

CREATE INDEX idx_models_provider ON models(provider);
CREATE INDEX idx_models_type ON models(model_type);

-- Agent-model assignments
CREATE TABLE agent_model_assignments (
    agent_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    assigned_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_sticky_brain BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (agent_id, model_id),
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    FOREIGN KEY (model_id) REFERENCES models(id) ON DELETE CASCADE
);

-- Audit log table
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id TEXT,
    agent_id TEXT,
    user_id TEXT,
    action_type TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    details_json TEXT,
    ip_address TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL
);

CREATE INDEX idx_audit_log_workspace ON audit_log(workspace_id);
CREATE INDEX idx_audit_log_created ON audit_log(created_at);
CREATE INDEX idx_audit_log_action ON audit_log(action_type);

-- File system cache
CREATE TABLE file_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    last_modified DATETIME NOT NULL,
    size INTEGER NOT NULL,
    cached_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    accessed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
    UNIQUE(workspace_id, file_path)
);

CREATE INDEX idx_file_cache_workspace ON file_cache(workspace_id);
CREATE INDEX idx_file_cache_hash ON file_cache(content_hash);

-- Settings table
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default settings
INSERT INTO settings (key, value_json) VALUES
    ('security.default_tier', '"auto"'),
    ('models.local_path', '"~/.jag/models"'),
    ('workspace.root_path', '"~/jag-workspaces"'),
    ('performance.max_concurrent_agents', '4'),
    ('performance.max_memory_mb', '4096');
```

### 7.2 Redis Cache Schema

```rust
// Redis key patterns
// workspace:{workspace_id}:agents -> SET of agent_ids
// agent:{agent_id}:state -> HASH {status, current_task, progress, last_heartbeat}
// agent:{agent_id}:tasks:queue -> LIST of task_ids
// task:{task_id}:status -> STRING {pending|running|completed|failed}
// artifact:{artifact_id}:content -> BINARY (for large artifacts)
// model:{model_id}:cache -> HASH {last_used, usage_count, avg_latency}
// rate_limit:{user_id}:{endpoint} -> COUNTER (for API rate limiting)
// session:{session_id} -> HASH {user_id, workspace_id, expires_at}

// Redis implementation example
use redis::{Client, Commands};

pub struct CacheManager {
    client: Client,
}

impl CacheManager {
    pub async fn update_agent_state(&self, agent_id: &str, state: &AgentState) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        let _: () = conn.hset_multiple(
            format!("agent:{}:state", agent_id),
            &[
                ("status", state.status.to_string()),
                ("current_task", state.current_task.clone().unwrap_or_default()),
                ("progress", state.progress.to_string()),
                ("last_heartbeat", Utc::now().to_rfc3339()),
            ],
        ).await?;
        
        // Set expiry (auto-cleanup if agent crashes)
        let _: () = conn.expire(format!("agent:{}:state", agent_id), 300).await?; // 5 minutes
        
        Ok(())
    }
    
    pub async fn get_agent_state(&self, agent_id: &str) -> Result<Option<AgentState>> {
        let mut conn = self.client.get_async_connection().await?;
        
        let (status, current_task, progress, last_heartbeat): 
            (String, String, String, String) = conn.hget_all(
            format!("agent:{}:state", agent_id)
        ).await?;
        
        Ok(Some(AgentState {
            status,
            current_task: if current_task.is_empty() { None } else { Some(current_task) },
            progress: progress.parse().unwrap_or(0),
            last_heartbeat: DateTime::parse_from_rfc3339(&last_heartbeat).ok(),
        }))
    }
    
    pub async fn push_task_to_queue(&self, agent_id: &str, task_id: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        let _: () = conn.rpush(
            format!("agent:{}:tasks:queue", agent_id),
            task_id,
        ).await?;
        
        Ok(())
    }
    
    pub async fn cache_artifact_content(&self, artifact_id: &str, content: &[u8]) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        let _: () = conn.set(
            format!("artifact:{}:content", artifact_id),
            content,
        ).await?;
        
        // Set expiry based on size (larger artifacts expire sooner)
        let expiry = if content.len() > 10_000_000 { 3600 } else { 86400 }; // 1 hour or 1 day
        let _: () = conn.expire(format!("artifact:{}:content", artifact_id), expiry).await?;
        
        Ok(())
    }
}
```

---

## 8. MODEL INTEGRATION ARCHITECTURE

### 8.1 Ollama Integration

```rust
// src/models/ollama_client.rs
use reqwest::Client;
use serde::{Serialize, Deserialize};

pub struct OllamaClient {
    base_url: String,
    client: Client,
}

#[derive(Serialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub options: Option<OllamaOptions>,
}

#[derive(Serialize)]
pub struct OllamaOptions {
    pub temperature: f32,
    pub top_p: f32,
    pub num_predict: i32,
    pub num_ctx: i32,
}

#[derive(Deserialize)]
pub struct OllamaGenerateResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
    pub prompt_eval_count: i32,
    pub eval_count: i32,
    pub total_duration: i64,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }
    
    pub async fn generate(&self, request: OllamaGenerateRequest) -> Result<OllamaGenerateResponse> {
        let url = format!("{}/api/generate", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<OllamaGenerateResponse>()
            .await?;
        
        Ok(response)
    }
    
    pub async fn pull_model(&self, model_name: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.base_url);
        
        let mut stream = self.client
            .post(&url)
            .json(&serde_json::json!({
                "name": model_name,
                "stream": true,
            }))
            .send()
            .await?
            .bytes_stream();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let progress: OllamaPullProgress = serde_json::from_slice(&chunk)?;
            
            // Log progress
            tracing::info!("Pulling model: {}% complete", progress.completed / progress.total * 100);
        }
        
        Ok(())
    }
    
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<OllamaModelList>()
            .await?;
        
        Ok(response.models)
    }
}

#[derive(Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: i64,
    pub digest: String,
    pub modified_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct OllamaModelList {
    pub models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
pub struct OllamaPullProgress {
    pub status: String,
    pub completed: i64,
    pub total: i64,
    pub digest: String,
}
```

### 8.2 Cloud Model Integration

```rust
// src/models/cloud_providers.rs
use reqwest::Client;
use serde::{Serialize, Deserialize};

pub struct CloudModelProvider {
    name: String,
    api_key: String,
    base_url: String,
    client: Client,
}

pub enum CloudProvider {
    Anthropic,
    OpenAI,
    Google,
    Mistral,
}

impl CloudModelProvider {
    pub fn new(provider: CloudProvider, api_key: &str) -> Self {
        let (name, base_url) = match provider {
            CloudProvider::Anthropic => ("Anthropic", "https://api.anthropic.com/v1"),
            CloudProvider::OpenAI => ("OpenAI", "https://api.openai.com/v1"),
            CloudProvider::Google => ("Google", "https://generativelanguage.googleapis.com/v1"),
            CloudProvider::Mistral => ("Mistral", "https://api.mistral.ai/v1"),
        };
        
        Self {
            name: name.to_string(),
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }
    
    pub async fn generate(&self, request: CloudGenerationRequest) -> Result<CloudGenerationResponse> {
        match self.name.as_str() {
            "Anthropic" => self.claude_generate(request).await,
            "OpenAI" => self.gpt_generate(request).await,
            "Google" => self.gemini_generate(request).await,
            "Mistral" => self.mistral_generate(request).await,
            _ => Err(Error::UnknownProvider),
        }
    }
    
    async fn claude_generate(&self, request: CloudGenerationRequest) -> Result<CloudGenerationResponse> {
        let url = format!("{}/messages", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": request.model,
                "messages": request.messages,
                "max_tokens": request.max_tokens,
                "temperature": request.temperature,
            }))
            .send()
            .await?
            .json::<ClaudeResponse>()
            .await?;
        
        Ok(CloudGenerationResponse {
            text: response.content[0].text.clone(),
            usage: TokenUsage {
                prompt_tokens: response.usage.input_tokens,
                completion_tokens: response.usage.output_tokens,
                total_tokens: response.usage.input_tokens + response.usage.output_tokens,
            },
        })
    }
    
    async fn gpt_generate(&self, request: CloudGenerationRequest) -> Result<CloudGenerationResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": request.model,
                "messages": request.messages,
                "max_tokens": request.max_tokens,
                "temperature": request.temperature,
            }))
            .send()
            .await?
            .json::<OpenAIResponse>()
            .await?;
        
        Ok(CloudGenerationResponse {
            text: response.choices[0].message.content.clone(),
            usage: TokenUsage {
                prompt_tokens: response.usage.prompt_tokens,
                completion_tokens: response.usage.completion_tokens,
                total_tokens: response.usage.total_tokens,
            },
        })
    }
}

#[derive(Serialize)]
pub struct CloudGenerationRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: i32,
    pub temperature: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ClaudeResponse {
    pub content: Vec<ContentBlock>,
    pub usage: ClaudeUsage,
}

#[derive(Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct ClaudeUsage {
    pub input_tokens: i32,
    pub output_tokens: i32,
}

#[derive(Deserialize)]
pub struct OpenAIResponse {
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
}

#[derive(Deserialize)]
pub struct OpenAIChoice {
    pub message: OpenAIMessage,
}

#[derive(Deserialize)]
pub struct OpenAIMessage {
    pub content: String,
}

#[derive(Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}
```

### 8.3 Model Context Management

```rust
// src/models/context_manager.rs
use tiktoken_rs::get_bpe_from_model;

pub struct ContextManager {
    max_context_window: usize,
    reserved_tokens: usize,
    tokenizer: tiktoken_rs::CoreBPE,
}

pub struct ManagedContext {
    pub messages: Vec<Message>,
    pub current_tokens: usize,
    pub system_prompt: Option<String>,
}

impl ContextManager {
    pub fn new(model_name: &str) -> Result<Self> {
        let tokenizer = get_bpe_from_model(model_name)
            .map_err(|_| Error::TokenizerNotFound)?;
        
        Ok(Self {
            max_context_window: Self::get_context_window(model_name),
            reserved_tokens: 1000, // Reserve for system prompt and safety margin
            tokenizer,
        })
    }
    
    pub fn add_message(&self, context: &mut ManagedContext, message: Message) -> Result<()> {
        let message_tokens = self.count_tokens(&message.content);
        
        if context.current_tokens + message_tokens > self.available_tokens() {
            // Need to truncate or remove old messages
            self.truncate_context(context, message_tokens)?;
        }
        
        context.messages.push(message);
        context.current_tokens += message_tokens;
        
        Ok(())
    }
    
    pub fn truncate_context(&self, context: &mut ManagedContext, needed_tokens: usize) -> Result<()> {
        // Remove oldest messages until we have space
        while context.current_tokens + needed_tokens > self.available_tokens() && !context.messages.is_empty() {
            let old_message = context.messages.remove(0);
            let old_tokens = self.count_tokens(&old_message.content);
            context.current_tokens = context.current_tokens.saturating_sub(old_tokens);
        }
        
        if context.current_tokens + needed_tokens > self.available_tokens() {
            return Err(Error::ContextWindowExceeded);
        }
        
        Ok(())
    }
    
    pub fn count_tokens(&self, text: &str) -> usize {
        self.tokenizer.encode_with_special_tokens(text).len()
    }
    
    fn available_tokens(&self) -> usize {
        self.max_context_window - self.reserved_tokens
    }
    
    fn get_context_window(model_name: &str) -> usize {
        match model_name {
            "gpt-4" => 8192,
            "gpt-4-turbo" => 128000,
            "claude-3-opus" => 200000,
            "claude-3-sonnet" => 200000,
            "gemma-7b" => 8192,
            "llama-3-70b" => 8192,
            _ => 4096,
        }
    }
}
```

---

## 9. SECURITY ARCHITECTURE

### 9.1 Security Layers

```rust
// src/security/mod.rs
pub struct SecurityLayer {
    authentication: AuthenticationLayer,
    authorization: AuthorizationLayer,
    sandbox: SandboxLayer,
    audit: AuditLayer,
    encryption: EncryptionLayer,
}

pub struct AuthenticationLayer {
    jwt_secret: String,
    session_store: RedisSessionStore,
}

pub struct AuthorizationLayer {
    rbac_engine: RBACEngine,
    permission_cache: PermissionCache,
}

pub struct SandboxLayer {
    seccomp_profile: SeccompProfile,
    namespace_isolator: NamespaceIsolator,
    resource_limiter: ResourceLimiter,
}

impl SecurityLayer {
    pub async fn authenticate_request(&self, request: &Request) -> Result<AuthenticatedUser> {
        let token = request.headers.get("Authorization")
            .ok_or(Error::MissingAuthToken)?;
        
        let claims = self.authentication.verify_jwt(token)?;
        
        // Check session validity
        let session = self.authentication.session_store
            .get_session(&claims.session_id)
            .await?
            .ok_or(Error::SessionExpired)?;
        
        Ok(AuthenticatedUser {
            user_id: claims.user_id,
            session_id: claims.session_id,
            roles: session.roles,
        })
    }
    
    pub async fn authorize_action(&self, user: &AuthenticatedUser, action: &Action, resource: &Resource) -> Result<()> {
        // Check RBAC permissions
        let allowed = self.authorization.rbac_engine
            .check_permission(&user.roles, action, resource)
            .await?;
        
        if !allowed {
            return Err(Error::PermissionDenied);
        }
        
        Ok(())
    }
    
    pub async fn execute_in_sandbox(&self, command: CommandSpec, user: &AuthenticatedUser) -> Result<CommandOutput> {
        // Apply seccomp filters
        self.sandbox.seccomp_profile.apply(&command)?;
        
        // Create isolated namespace
        let namespace = self.sandbox.namespace_isolator.create_namespace(user.user_id)?;
        
        // Set resource limits
        self.sandbox.resource_limiter.apply_limits(&command)?;
        
        // Execute command
        let output = command.execute_in_namespace(&namespace).await?;
        
        // Clean up namespace
        self.sandbox.namespace_isolator.destroy_namespace(&namespace)?;
        
        Ok(output)
    }
}

// JWT Token Structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JWTClaims {
    pub sub: String,           // User ID
    pub session_id: String,
    pub workspace_id: Option<String>,
    pub roles: Vec<String>,
    pub exp: i64,              // Expiration time
    pub iat: i64,              // Issued at
    pub iss: String,           // Issuer
}

// RBAC Permission System
pub struct RBACEngine {
    db_pool: SqlitePool,
}

impl RBACEngine {
    pub async fn check_permission(&self, roles: &[String], action: &Action, resource: &Resource) -> Result<bool> {
        // Query database for role permissions
        let permissions: Vec<Permission> = sqlx::query_as!(
            Permission,
            r#"
            SELECT p.*
            FROM permissions p
            JOIN role_permissions rp ON p.id = rp.permission_id
            JOIN roles r ON r.id = rp.role_id
            WHERE r.name = ANY(?)
              AND p.resource_type = ?
              AND p.action = ?
            "#,
            roles,
            resource.resource_type,
            action.to_string(),
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(!permissions.is_empty())
    }
}

// Seccomp Profile for Linux
#[cfg(target_os = "linux")]
pub struct SeccompProfile {
    rules: Vec<SeccompRule>,
}

#[cfg(target_os = "linux")]
impl SeccompProfile {
    pub fn apply(&self, command: &CommandSpec) -> Result<()> {
        use libseccomp::*;
        
        let mut ctx = ScmpFilterContext::new(ScmpAction::Errno(libc::EPERM))?;
        
        // Allow basic syscalls
        let allowed_syscalls = vec![
            ScmpSyscall::from_name("read")?,
            ScmpSyscall::from_name("write")?,
            ScmpSyscall::from_name("open")?,
            ScmpSyscall::from_name("close")?,
            ScmpSyscall::from_name("stat")?,
            ScmpSyscall::from_name("fstat")?,
            ScmpSyscall::from_name("mmap")?,
            ScmpSyscall::from_name("mprotect")?,
            ScmpSyscall::from_name("munmap")?,
            ScmpSyscall::from_name("brk")?,
            ScmpSyscall::from_name("rt_sigprocmask")?,
            ScmpSyscall::from_name("ioctl")?,
            ScmpSyscall::from_name("access")?,
            ScmpSyscall::from_name("pipe")?,
            ScmpSyscall::from_name("select")?,
            ScmpSyscall::from_name("sched_yield")?,
            ScmpSyscall::from_name("mremap")?,
            ScmpSyscall::from_name("msync")?,
            ScmpSyscall::from_name("mincore")?,
            ScmpSyscall::from_name("madvise")?,
            ScmpSyscall::from_name("dup")?,
            ScmpSyscall::from_name("nanosleep")?,
            ScmpSyscall::from_name("getpid")?,
            ScmpSyscall::from_name("socket")?,
            ScmpSyscall::from_name("connect")?,
            ScmpSyscall::from_name("sendto")?,
            ScmpSyscall::from_name("recvfrom")?,
            ScmpSyscall::from_name("shutdown")?,
            ScmpSyscall::from_name("bind")?,
            ScmpSyscall::from_name("listen")?,
            ScmpSyscall::from_name("getsockname")?,
            ScmpSyscall::from_name("getpeername")?,
            ScmpSyscall::from_name("socketpair")?,
            ScmpSyscall::from_name("clone")?,
            ScmpSyscall::from_name("fork")?,
            ScmpSyscall::from_name("vfork")?,
            ScmpSyscall::from_name("execve")?,
            ScmpSyscall::from_name("exit")?,
            ScmpSyscall::from_name("wait4")?,
            ScmpSyscall::from_name("kill")?,
            ScmpSyscall::from_name("uname")?,
            ScmpSyscall::from_name("fcntl")?,
            ScmpSyscall::from_name("flock")?,
            ScmpSyscall::from_name("fsync")?,
            ScmpSyscall::from_name("fdatasync")?,
            ScmpSyscall::from_name("truncate")?,
            ScmpSyscall::from_name("ftruncate")?,
            ScmpSyscall::from_name("getdents")?,
            ScmpSyscall::from_name("getcwd")?,
            ScmpSyscall::from_name("chdir")?,
            ScmpSyscall::from_name("rename")?,
            ScmpSyscall::from_name("mkdir")?,
            ScmpSyscall::from_name("rmdir")?,
            ScmpSyscall::from_name("creat")?,
            ScmpSyscall::from_name("link")?,
            ScmpSyscall::from_name("unlink")?,
            ScmpSyscall::from_name("symlink")?,
            ScmpSyscall::from_name("readlink")?,
            ScmpSyscall::from_name("chmod")?,
            ScmpSyscall::from_name("fchmod")?,
            ScmpSyscall::from_name("chown")?,
            ScmpSyscall::from_name("fchown")?,
            ScmpSyscall::from_name("umask")?,
            ScmpSyscall::from_name("gettimeofday")?,
            ScmpSyscall::from_name("getrlimit")?,
            ScmpSyscall::from_name("getrusage")?,
            ScmpSyscall::from_name("sysinfo")?,
            ScmpSyscall::from_name("times")?,
            ScmpSyscall::from_name("getuid")?,
            ScmpSyscall::from_name("getgid")?,
            ScmpSyscall::from_name("geteuid")?,
            ScmpSyscall::from_name("getegid")?,
            ScmpSyscall::from_name("setpgid")?,
            ScmpSyscall::from_name("getppid")?,
            ScmpSyscall::from_name("getpgrp")?,
            ScmpSyscall::from_name("setsid")?,
            ScmpSyscall::from_name("setreuid")?,
            ScmpSyscall::from_name("setregid")?,
            ScmpSyscall::from_name("getgroups")?,
            ScmpSyscall::from_name("setgroups")?,
            ScmpSyscall::from_name("setresuid")?,
            ScmpSyscall::from_name("getresuid")?,
            ScmpSyscall::from_name("setresgid")?,
            ScmpSyscall::from_name("getresgid")?,
            ScmpSyscall::from_name("getpgid")?,
            ScmpSyscall::from_name("setfsuid")?,
            ScmpSyscall::from_name("setfsgid")?,
            ScmpSyscall::from_name("getsid")?,
            ScmpSyscall::from_name("capget")?,
            ScmpSyscall::from_name("capset")?,
            ScmpSyscall::from_name("rt_sigpending")?,
            ScmpSyscall::from_name("rt_sigtimedwait")?,
            ScmpSyscall::from_name("rt_sigqueueinfo")?,
            ScmpSyscall::from_name("rt_sigsuspend")?,
            ScmpSyscall::from_name("sigaltstack")?,
            ScmpSyscall::from_name("utime")?,
            ScmpSyscall::from_name("mknod")?,
            ScmpSyscall::from_name("personality")?,
            ScmpSyscall::from_name("statfs")?,
            ScmpSyscall::from_name("fstatfs")?,
            ScmpSyscall::from_name("sysfs")?,
            ScmpSyscall::from_name("getpriority")?,
            ScmpSyscall::from_name("setpriority")?,
            ScmpSyscall::from_name("sched_setparam")?,
            ScmpSyscall::from_name("sched_getparam")?,
            ScmpSyscall::from_name("sched_setscheduler")?,
            ScmpSyscall::from_name("sched_getscheduler")?,
            ScmpSyscall::from_name("sched_get_priority_max")?,
            ScmpSyscall::from_name("sched_get_priority_min")?,
            ScmpSyscall::from_name("sched_rr_get_interval")?,
            ScmpSyscall::from_name("mlock")?,
            ScmpSyscall::from_name("munlock")?,
            ScmpSyscall::from_name("mlockall")?,
            ScmpSyscall::from_name("munlockall")?,
            ScmpSyscall::from_name("vhangup")?,
            ScmpSyscall::from_name("pivot_root")?,
            ScmpSyscall::from_name("prctl")?,
            ScmpSyscall::from_name("arch_prctl")?,
            ScmpSyscall::from_name("adjtimex")?,
            ScmpSyscall::from_name("setrlimit")?,
            ScmpSyscall::from_name("chroot")?,
            ScmpSyscall::from_name("sync")?,
            ScmpSyscall::from_name("acct")?,
            ScmpSyscall::from_name("settimeofday")?,
            ScmpSyscall::from_name("mount")?,
            ScmpSyscall::from_name("umount2")?,
            ScmpSyscall::from_name("swapon")?,
            ScmpSyscall::from_name("swapoff")?,
            ScmpSyscall::from_name("reboot")?,
            ScmpSyscall::from_name("sethostname")?,
            ScmpSyscall::from_name("setdomainname")?,
            ScmpSyscall::from_name("ioperm")?,
            ScmpSyscall::from_name("create_module")?,
            ScmpSyscall::from_name("init_module")?,
            ScmpSyscall::from_name("delete_module")?,
            ScmpSyscall::from_name("get_kernel_syms")?,
            ScmpSyscall::from_name("query_module")?,
            ScmpSyscall::from_name("quotactl")?,
            ScmpSyscall::from_name("nfsservctl")?,
            ScmpSyscall::from_name("getpmsg")?,
            ScmpSyscall::from_name("putpmsg")?,
            ScmpSyscall::from_name("afs_syscall")?,
            ScmpSyscall::from_name("tuxcall")?,
            ScmpSyscall::from_name("security")?,
            ScmpSyscall::from_name("gettid")?,
            ScmpSyscall::from_name("readahead")?,
            ScmpSyscall::from_name("setxattr")?,
            ScmpSyscall::from_name("lsetxattr")?,
            ScmpSyscall::from_name("fsetxattr")?,
            ScmpSyscall::from_name("getxattr")?,
            ScmpSyscall::from_name("lgetxattr")?,
            ScmpSyscall::from_name("fgetxattr")?,
            ScmpSyscall::from_name("listxattr")?,
            ScmpSyscall::from_name("llistxattr")?,
            ScmpSyscall::from_name("flistxattr")?,
            ScmpSyscall::from_name("removexattr")?,
            ScmpSyscall::from_name("lremovexattr")?,
            ScmpSyscall::from_name("fremovexattr")?,
            ScmpSyscall::from_name("tkill")?,
            ScmpSyscall::from_name("time")?,
            ScmpSyscall::from_name("futex")?,
            ScmpSyscall::from_name("sched_setaffinity")?,
            ScmpSyscall::from_name("sched_getaffinity")?,
            ScmpSyscall::from_name("set_thread_area")?,
            ScmpSyscall::from_name("io_setup")?,
            ScmpSyscall::from_name("io_destroy")?,
            ScmpSyscall::from_name("io_getevents")?,
            ScmpSyscall::from_name("io_submit")?,
            ScmpSyscall::from_name("io_cancel")?,
            ScmpSyscall::from_name("get_thread_area")?,
            ScmpSyscall::from_name("lookup_dcookie")?,
            ScmpSyscall::from_name("epoll_create")?,
            ScmpSyscall::from_name("epoll_ctl_old")?,
            ScmpSyscall::from_name("epoll_wait_old")?,
            ScmpSyscall::from_name("remap_file_pages")?,
            ScmpSyscall::from_name("getdents64")?,
            ScmpSyscall::from_name("set_tid_address")?,
            ScmpSyscall::from_name("restart_syscall")?,
            ScmpSyscall::from_name("semtimedop")?,
            ScmpSyscall::from_name("fadvise64")?,
            ScmpSyscall::from_name("timer_create")?,
            ScmpSyscall::from_name("timer_settime")?,
            ScmpSyscall::from_name("timer_gettime")?,
            ScmpSyscall::from_name("timer_getoverrun")?,
            ScmpSyscall::from_name("timer_delete")?,
            ScmpSyscall::from_name("clock_settime")?,
            ScmpSyscall::from_name("clock_gettime")?,
            ScmpSyscall::from_name("clock_getres")?,
            ScmpSyscall::from_name("clock_nanosleep")?,
            ScmpSyscall::from_name("exit_group")?,
            ScmpSyscall::from_name("epoll_wait")?,
            ScmpSyscall::from_name("epoll_ctl")?,
            ScmpSyscall::from_name("tgkill")?,
            ScmpSyscall::from_name("utimes")?,
            ScmpSyscall::from_name("vserver")?,
            ScmpSyscall::from_name("mbind")?,
            ScmpSyscall::from_name("set_mempolicy")?,
            ScmpSyscall::from_name("get_mempolicy")?,
            ScmpSyscall::from_name("mq_open")?,
            ScmpSyscall::from_name("mq_unlink")?,
            ScmpSyscall::from_name("mq_timedsend")?,
            ScmpSyscall::from_name("mq_timedreceive")?,
            ScmpSyscall::from_name("mq_notify")?,
            ScmpSyscall::from_name("mq_getsetattr")?,
            ScmpSyscall::from_name("kexec_load")?,
            ScmpSyscall::from_name("waitid")?,
            ScmpSyscall::from_name("add_key")?,
            ScmpSyscall::from_name("request_key")?,
            ScmpSyscall::from_name("keyctl")?,
            ScmpSyscall::from_name("ioprio_set")?,
            ScmpSyscall::from_name("ioprio_get")?,
            ScmpSyscall::from_name("inotify_init")?,
            ScmpSyscall::from_name("inotify_add_watch")?,
            ScmpSyscall::from_name("inotify_rm_watch")?,
            ScmpSyscall::from_name("migrate_pages")?,
            ScmpSyscall::from_name("openat")?,
            ScmpSyscall::from_name("mkdirat")?,
            ScmpSyscall::from_name("mknodat")?,
            ScmpSyscall::from_name("fchownat")?,
            ScmpSyscall::from_name("futimesat")?,
            ScmpSyscall::from_name("newfstatat")?,
            ScmpSyscall::from_name("unlinkat")?,
            ScmpSyscall::from_name("renameat")?,
            ScmpSyscall::from_name("linkat")?,
            ScmpSyscall::from_name("symlinkat")?,
            ScmpSyscall::from_name("readlinkat")?,
            ScmpSyscall::from_name("fchmodat")?,
            ScmpSyscall::from_name("faccessat")?,
            ScmpSyscall::from_name("pselect6")?,
            ScmpSyscall::from_name("ppoll")?,
            ScmpSyscall::from_name("unshare")?,
            ScmpSyscall::from_name("set_robust_list")?,
            ScmpSyscall::from_name("get_robust_list")?,
            ScmpSyscall::from_name("splice")?,
            ScmpSyscall::from_name("tee")?,
            ScmpSyscall::from_name("sync_file_range")?,
            ScmpSyscall::from_name("vmsplice")?,
            ScmpSyscall::from_name("move_pages")?,
            ScmpSyscall::from_name("utimensat")?,
            ScmpSyscall::from_name("epoll_pwait")?,
            ScmpSyscall::from_name("signalfd")?,
            ScmpSyscall::from_name("timerfd_create")?,
            ScmpSyscall::from_name("eventfd")?,
            ScmpSyscall::from_name("fallocate")?,
            ScmpSyscall::from_name("timerfd_settime")?,
            ScmpSyscall::from_name("timerfd_gettime")?,
            ScmpSyscall::from_name("accept4")?,
            ScmpSyscall::from_name("signalfd4")?,
            ScmpSyscall::from_name("eventfd2")?,
            ScmpSyscall::from_name("epoll_create1")?,
            ScmpSyscall::from_name("dup3")?,
            ScmpSyscall::from_name("pipe2")?,
            ScmpSyscall::from_name("inotify_init1")?,
            ScmpSyscall::from_name("preadv")?,
            ScmpSyscall::from_name("pwritev")?,
            ScmpSyscall::from_name("rt_tgsigqueueinfo")?,
            ScmpSyscall::from_name("perf_event_open")?,
            ScmpSyscall::from_name("recvmmsg")?,
            ScmpSyscall::from_name("fanotify_init")?,
            ScmpSyscall::from_name("fanotify_mark")?,
            ScmpSyscall::from_name("prlimit64")?,
            ScmpSyscall::from_name("name_to_handle_at")?,
            ScmpSyscall::from_name("open_by_handle_at")?,
            ScmpSyscall::from_name("clock_adjtime")?,
            ScmpSyscall::from_name("syncfs")?,
            ScmpSyscall::from_name("sendmmsg")?,
            ScmpSyscall::from_name("setns")?,
            ScmpSyscall::from_name("getcpu")?,
            ScmpSyscall::from_name("process_vm_readv")?,
            ScmpSyscall::from_name("process_vm_writev")?,
            ScmpSyscall::from_name("kcmp")?,
            ScmpSyscall::from_name("finit_module")?,
            ScmpSyscall::from_name("sched_setattr")?,
            ScmpSyscall::from_name("sched_getattr")?,
            ScmpSyscall::from_name("renameat2")?,
            ScmpSyscall::from_name("seccomp")?,
            ScmpSyscall::from_name("getrandom")?,
            ScmpSyscall::from_name("memfd_create")?,
            ScmpSyscall::from_name("kexec_file_load")?,
            ScmpSyscall::from_name("bpf")?,
            ScmpSyscall::from_name("execveat")?,
            ScmpSyscall::from_name("userfaultfd")?,
            ScmpSyscall::from_name("membarrier")?,
            ScmpSyscall::from_name("mlock2")?,
            ScmpSyscall::from_name("copy_file_range")?,
            ScmpSyscall::from_name("preadv2")?,
            ScmpSyscall::from_name("pwritev2")?,
            ScmpSyscall::from_name("pkey_mprotect")?,
            ScmpSyscall::from_name("pkey_alloc")?,
            ScmpSyscall::from_name("pkey_free")?,
            ScmpSyscall::from_name("statx")?,
            ScmpSyscall::from_name("io_pgetevents")?,
            ScmpSyscall::from_name("rseq")?,
        ];
        
        for syscall in allowed_syscalls {
            ctx.add_rule(&ScmpArgCompare::new(0, ScmpCompareOp::Equal, 0), syscall)?;
        }
        
        // Load the filter
        ctx.load()?;
        
        Ok(())
    }
}
```

### 9.2 Encryption & Data Protection

```rust
// src/security/encryption.rs
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub struct EncryptionLayer {
    master_key: Aes256Gcm,
}

impl EncryptionLayer {
    pub fn new(master_key: &[u8; 32]) -> Self {
        Self {
            master_key: Aes256Gcm::new_from_slice(master_key).unwrap(),
        }
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        
        let ciphertext = self.master_key
            .encrypt(&Nonce::from_slice(&nonce), plaintext)
            .map_err(|_| Error::EncryptionFailed)?;
        
        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(Error::InvalidCiphertext);
        }
        
        let (nonce, ciphertext) = ciphertext.split_at(12);
        
        let plaintext = self.master_key
            .decrypt(&Nonce::from_slice(nonce), ciphertext)
            .map_err(|_| Error::DecryptionFailed)?;
        
        Ok(plaintext)
    }
    
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| Error::PasswordHashFailed)?
            .to_string();
        
        Ok(password_hash)
    }
    
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| Error::InvalidPasswordHash)?;
        
        let is_valid = Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();
        
        Ok(is_valid)
    }
}
```

---

## 10. PERFORMANCE REQUIREMENTS

### 10.1 Performance Targets

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **IDE Startup Time** | <2 seconds | Cold start to interactive |
| **Agent Spin-up Time** | <800ms | From request to ready state |
| **Model Response Time (Local)** | <2s (p50), <5s (p95) | Ollama generation latency |
| **Model Response Time (Cloud)** | <3s (p50), <8s (p95) | Cloud API latency |
| **File Open (1MB)** | <100ms | Time to load and render |
| **Search (100k files)** | <500ms | Full-text search latency |
| **Code Completion** | <100ms | LSP/IntelliSense response |
| **Artifact Generation** | <5s | From task completion to artifact ready |
| **Memory Usage (Idle)** | <300MB | RSS memory when idle |
| **Memory Usage (Active)** | <1.5GB | RSS with 4 active agents |
| **CPU Usage** | <30% (avg) | During normal operation |
| **Disk I/O** | <50MB/s | Read/write throughput |
| **Concurrent Agents** | 5+ | Simultaneous agent execution |
| **WebSocket Latency** | <50ms | Real-time update delivery |

### 10.2 Performance Optimization Strategies

```rust
// src/performance/optimizations.rs

// 1. Lazy Loading
pub struct LazyLoader {
    cache: LruCache<String, Vec<u8>>,
    db_pool: SqlitePool,
}

impl LazyLoader {
    pub async fn load_file(&self, path: &Path) -> Result<Vec<u8>> {
        // Check cache first
        if let Some(content) = self.cache.get(path.to_str().unwrap()) {
            return Ok(content.clone());
        }
        
        // Load from disk
        let content = tokio::fs::read(path).await?;
        
        // Cache for future use
        self.cache.put(path.to_str().unwrap().to_string(), content.clone());
        
        Ok(content)
    }
}

// 2. Connection Pooling
use bb8::Pool;
use bb8_redis::{RedisConnectionManager, RedisPool};

pub struct ConnectionPoolManager {
    redis_pool: RedisPool,
    db_pool: SqlitePool,
}

impl ConnectionPoolManager {
    pub fn new(redis_url: &str, db_path: &str) -> Result<Self> {
        let redis_manager = RedisConnectionManager::new(redis_url)?;
        let redis_pool = Pool::builder()
            .max_size(20)
            .min_idle(Some(5))
            .max_lifetime(Some(std::time::Duration::from_secs(300)))
            .build(redis_manager)?;
        
        let db_pool = SqlitePool::connect(db_path)?;
        
        Ok(Self {
            redis_pool,
            db_pool,
        })
    }
}

// 3. Parallel Processing
use rayon::prelude::*;

pub struct ParallelProcessor;

impl ParallelProcessor {
    pub fn process_files<F, T>(files: Vec<PathBuf>, processor: F) -> Vec<T>
    where
        F: Fn(PathBuf) -> T + Send + Sync,
        T: Send,
    {
        files.par_iter()
            .map(|file| processor(file.clone()))
            .collect()
    }
    
    pub async fn process_agents_parallel(agents: Vec<AgentId>, orchestrator: &AgentOrchestrator) -> Vec<Result<()>> {
        let futures: Vec<_> = agents
            .iter()
            .map(|agent_id| orchestrator.process_agent(*agent_id))
            .collect();
        
        futures::future::join_all(futures).await
    }
}

// 4. Caching Strategy
use moka::future::Cache;

pub struct CacheStrategy {
    code_cache: Cache<String, String>,
    model_cache: Cache<String, String>,
    artifact_cache: Cache<String, Vec<u8>>,
}

impl CacheStrategy {
    pub fn new() -> Self {
        Self {
            code_cache: Cache::builder()
                .max_capacity(1000)
                .time_to_live(std::time::Duration::from_secs(3600))
                .build(),
            model_cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(std::time::Duration::from_secs(1800))
                .build(),
            artifact_cache: Cache::builder()
                .max_capacity(500)
                .time_to_live(std::time::Duration::from_secs(7200))
                .build(),
        }
    }
    
    pub async fn get_code(&self, key: &str) -> Option<String> {
        self.code_cache.get(key).await
    }
    
    pub async fn set_code(&self, key: String, value: String) {
        self.code_cache.insert(key, value).await;
    }
}

// 5. Memory Management
pub struct MemoryManager {
    max_memory_mb: u64,
    current_memory_mb: AtomicU64,
}

impl MemoryManager {
    pub fn new(max_memory_mb: u64) -> Self {
        Self {
            max_memory_mb,
            current_memory_mb: AtomicU64::new(0),
        }
    }
    
    pub fn can_allocate(&self, size_mb: u64) -> bool {
        let current = self.current_memory_mb.load(Ordering::Relaxed);
        current + size_mb <= self.max_memory_mb
    }
    
    pub fn allocate(&self, size_mb: u64) -> Result<()> {
        if !self.can_allocate(size_mb) {
            return Err(Error::OutOfMemory);
        }
        
        self.current_memory_mb.fetch_add(size_mb, Ordering::Relaxed);
        Ok(())
    }
    
    pub fn deallocate(&self, size_mb: u64) {
        self.current_memory_mb.fetch_sub(size_mb, Ordering::Relaxed);
    }
    
    pub fn get_usage(&self) -> u64 {
        self.current_memory_mb.load(Ordering::Relaxed)
    }
}

// 6. Async I/O Optimization
use tokio::io::AsyncReadExt;

pub struct AsyncIOOptimizer;

impl AsyncIOOptimizer {
    pub async fn read_file_chunked(path: &Path, chunk_size: usize) -> Result<Vec<u8>> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        let mut chunk = vec![0u8; chunk_size];
        
        loop {
            let bytes_read = file.read(&mut chunk).await?;
            if bytes_read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..bytes_read]);
        }
        
        Ok(buffer)
    }
    
    pub async fn write_file_buffered(path: &Path, data: &[u8], buffer_size: usize) -> Result<()> {
        let mut file = tokio::fs::File::create(path).await?;
        
        for chunk in data.chunks(buffer_size) {
            use tokio::io::AsyncWriteExt;
            file.write_all(chunk).await?;
        }
        
        file.flush().await?;
        Ok(())
    }
}
```

---

## 11. IMPLEMENTATION PLAN

### 11.1 Phase 1: Foundation (Months 1-3)

**Week 1-4: Core Infrastructure**
- [ ] Set up VS Code OSS fork
- [ ] Implement Rust/Electron bridge
- [ ] Create basic project structure
- [ ] Set up build system (Cargo + Webpack)
- [ ] Implement SQLite database schema
- [ ] Set up Redis cache
- [ ] Create basic UI shell

**Week 5-8: Agent Engine**
- [ ] Implement Agent Orchestrator
- [ ] Create A2A communication protocol
- [ ] Build workflow engine (DAG)
- [ ] Implement task queue system
- [ ] Create artifact generator
- [ ] Set up logging and monitoring

**Week 9-12: Model Integration**
- [ ] Integrate Ollama runtime
- [ ] Implement model router
- [ ] Add cloud model support (Claude, GPT)
- [ ] Create context manager
- [ ] Implement token counting and limits
- [ ] Build model download/management system

**Deliverables:**
- Working IDE with single AI assistant
- Basic agent orchestration
- Local and cloud model support
- Database and cache infrastructure

### 11.2 Phase 2: Multi-Agent System (Months 4-6)

**Week 13-16: Specialized Agents**
- [ ] Implement Agent 1 (Planner)
- [ ] Implement Agent 2 (Backend)
- [ ] Implement Agent 3 (Frontend)
- [ ] Implement Agent 4 (Integration)
- [ ] Create agent specialization logic
- [ ] Build inter-agent communication

**Week 17-20: Execution Sandbox**
- [ ] Implement security policy engine
- [ ] Create terminal sandbox
- [ ] Build file system sandbox
- [ ] Implement resource limits
- [ ] Add seccomp profiles (Linux)
- [ ] Create audit logging system

**Week 21-24: Mission Control UI**
- [ ] Build agent dashboard
- [ ] Create workflow visualization
- [ ] Implement artifact viewer
- [ ] Add real-time status updates (WebSocket)
- [ ] Build task management interface
- [ ] Create approval workflow UI

**Deliverables:**
- Full 4-agent team system
- Security sandbox with 3 tiers
- Mission Control dashboard
- Artifact verification system

### 11.3 Phase 3: Advanced Features (Months 7-9)

**Week 25-28: Model Garden**
- [ ] Implement \"Sticky Brain\" selector
- [ ] Build model switching logic
- [ ] Create cost optimizer
- [ ] Add model performance metrics
- [ ] Implement fallback strategies
- [ ] Build model benchmarking system

**Week 29-32: Browser Sub-Agent**
- [ ] Integrate Playwright
- [ ] Implement browser automation
- [ ] Create UI testing framework
- [ ] Build screenshot generation
- [ ] Add DOM interaction capabilities
- [ ] Implement visual regression testing

**Week 33-36: Performance Optimization**
- [ ] Implement GPU rendering (wgpu)
- [ ] Optimize memory usage
- [ ] Add connection pooling
- [ ] Implement lazy loading
- [ ] Build caching strategies
- [ ] Profile and optimize hot paths

**Deliverables:**
- Hybrid model routing
- Browser automation
- GPU-accelerated UI
- Performance optimizations

### 11.4 Phase 4: Polish & Launch (Months 10-12)

**Week 37-40: Testing & QA**
- [ ] Write unit tests (80% coverage)
- [ ] Create integration tests
- [ ] Build E2E test suite
- [ ] Perform security audit
- [ ] Conduct performance testing
- [ ] Fix critical bugs

**Week 41-44: Documentation & SDK**
- [ ] Write user documentation
- [ ] Create API documentation
- [ ] Build extension SDK
- [ ] Write developer guides
- [ ] Create tutorial videos
- [ ] Set up community forums

**Week 45-48: Enterprise Features**
- [ ] Implement SSO (SAML/OAuth)
- [ ] Add team collaboration
- [ ] Build audit compliance tools
- [ ] Create cost controls
- [ ] Implement usage analytics
- [ ] Add admin dashboard

**Week 49-52: Launch Preparation**
- [ ] Beta testing program
- [ ] Marketing materials
- [ ] Community building
- [ ] Final bug fixes
- [ ] Performance tuning
- [ ] v1.0 release

**Deliverables:**
- Production-ready IDE
- Comprehensive documentation
- Extension ecosystem
- Enterprise features
- v1.0 launch

---

## 12. TESTING STRATEGY

### 12.1 Test Pyramid

```
            /E2E Tests\
           /-----------\
          /Integration  \
         /---------------\
        /  Unit Tests     \
       /-------------------\
```

### 12.2 Unit Testing

```rust
// tests/unit/agent_orchestrator.rs
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_spawn_agent_team() {
        let orchestrator = AgentOrchestrator::new().await.unwrap();
        
        let team_id = orchestrator.spawn_agent_team(ProjectId::new()).await.unwrap();
        
        assert!(orchestrator.teams.contains_key(&team_id));
        assert_eq!(orchestrator.teams.get(&team_id).unwrap().agents.len(), 4);
    }
    
    #[tokio::test]
    async fn test_dispatch_task_security_check() {
        let orchestrator = AgentOrchestrator::new().await.unwrap();
        let agent_id = orchestrator.create_agent(AgentRole::Planner).await.unwrap();
        
        // Set security tier to Off
        orchestrator.set_security_tier(agent_id, SecurityTier::Off).await.unwrap();
        
        let task = Task {
            task_type: TaskType::ModifyFile,
            payload: serde_json::json!({"path": "/test.txt"}),
        };
        
        let result = orchestrator.dispatch_task(agent_id, task).await;
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::ApprovalRequired);
    }
    
    #[test]
    fn test_workflow_dag_cycle_detection() {
        let mut workflow = WorkflowEngine::new();
        
        workflow.add_task(TaskId::new(), vec![]).unwrap();
        workflow.add_task(TaskId::new(), vec![TaskId::from(0)]).unwrap();
        
        // This should create a cycle
        let result = workflow.add_dependency(TaskId::from(0), TaskId::from(1));
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::CircularDependency);
    }
}

// tests/unit/model_router.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_model_selection_reasoning() {
        let router = ModelRouter::new().await.unwrap();
        
        let model = router.select_model(ModelPreference::Reasoning).await.unwrap();
        
        assert!(matches!(model, Model::Cloud(_)));
        assert!(model.provider().is_reasoning_capable());
    }
    
    #[tokio::test]
    async fn test_model_fallback() {
        let router = ModelRouter::new().await.unwrap();
        
        // Simulate cloud provider failure
        router.cloud_providers.write().await.clear();
        
        let model = router.select_model(ModelPreference::CodeGeneration).await.unwrap();
        
        assert!(matches!(model, Model::Local(_)));
    }
}
```

### 12.3 Integration Testing

```rust
// tests/integration/agent_workflow.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_agent_workflow() {
        // Setup
        let workspace = create_test_workspace().await;
        let orchestrator = AgentOrchestrator::new().await.unwrap();
        
        // Create agent team
        let team_id = orchestrator.spawn_agent_team(workspace.id).await.unwrap();
        
        // Dispatch planning task
        let task = Task {
            task_type: TaskType::GeneratePRD,
            payload: serde_json::json!({
                "project_name": \"Test App\",
                "description": \"A test application\"
            }),
        };
        
        let planner_id = orchestrator.get_agent_by_role(team_id, AgentRole::Planner).await.unwrap();
        orchestrator.dispatch_task(planner_id, task).await.unwrap();
        
        // Wait for completion
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        // Verify PRD artifact created
        let artifacts = orchestrator.get_agent_artifacts(planner_id).await.unwrap();
        let prd_artifact = artifacts.iter().find(|a| a.artifact_type == ArtifactType::PRD).unwrap();
        
        assert_eq!(prd_artifact.verification_status, VerificationStatus::Pending);
        
        // Approve artifact
        orchestrator.approve_artifact(prd_artifact.id).await.unwrap();
        
        // Verify backend agent started
        let backend_id = orchestrator.get_agent_by_role(team_id, AgentRole::Backend).await.unwrap();
        let backend_state = orchestrator.get_agent_state(backend_id).await.unwrap();
        
        assert_eq!(backend_state.status, AgentStatus::Working);
    }
    
    #[tokio::test]
    async fn test_multi_agent_collaboration() {
        let workspace = create_test_workspace().await;
        let orchestrator = AgentOrchestrator::new().await.unwrap();
        
        let team_id = orchestrator.spawn_agent_team(workspace.id).await.unwrap();
        
        // Dispatch tasks to multiple agents
        let tasks = vec![
            (AgentRole::Backend, TaskType::ImplementAPI),
            (AgentRole::Frontend, TaskType::BuildUI),
        ];
        
        for (role, task_type) in tasks {
            let agent_id = orchestrator.get_agent_by_role(team_id, role).await.unwrap();
            let task = Task {
                task_type,
                payload: serde_json::json!({}),
            };
            orchestrator.dispatch_task(agent_id, task).await.unwrap();
        }
        
        // Monitor progress
        let mut progress_updates = Vec::new();
        let mut rx = orchestrator.subscribe_to_team(team_id);
        
        while let Ok(update) = rx.recv().await {
            progress_updates.push(update);
            
            if progress_updates.len() >= 6 { // 2 agents × 3 updates each
                break;
            }
        }
        
        assert!(progress_updates.len() >= 6);
        
        // Verify both agents completed
        for (role, _) in tasks {
            let agent_id = orchestrator.get_agent_by_role(team_id, role).await.unwrap();
            let state = orchestrator.get_agent_state(agent_id).await.unwrap();
            assert_eq!(state.status, AgentStatus::Completed);
        }
    }
}
```

### 12.4 E2E Testing

```typescript
// tests/e2e/mission_control.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Mission Control', () => {
  test('should create agent team and monitor progress', async ({ page }) => {
    // Navigate to Mission Control
    await page.goto('http://localhost:3000/mission-control');
    
    // Create new agent team
    await page.click('[data-testid=\"create-team-button\"]');
    
    // Fill project details
    await page.fill('[data-testid=\"project-name\"]', 'Grocery App');
    await page.fill('[data-testid=\"project-description\"]', 'Build a grocery selling application');
    await page.selectOption('[data-testid=\"security-tier\"]', 'auto');
    
    // Submit
    await page.click('[data-testid=\"submit-button\"]');
    
    // Wait for team creation
    await expect(page.locator('[data-testid=\"team-created\"]')).toBeVisible();
    
    // Verify 4 agents are created
    const agents = page.locator('[data-testid=\"agent-card\"]');
    await expect(agents).toHaveCount(4);
    
    // Monitor Agent 1 (Planner) progress
    await expect(page.locator('[data-testid=\"agent-1-status\"]')).toHaveText('Working');
    
    // Wait for PRD artifact
    await page.waitForSelector('[data-testid=\"artifact-prd\"]', { timeout: 60000 });
    
    // Review and approve PRD
    await page.click('[data-testid=\"artifact-prd-view\"]');
    await page.click('[data-testid=\"approve-button\"]');
    
    // Verify Agent 2 (Backend) started
    await expect(page.locator('[data-testid=\"agent-2-status\"]')).toHaveText('Working');
    
    // Wait for all agents to complete
    await page.waitForSelector('[data-testid=\"all-agents-completed\"]', { timeout: 300000 });
    
    // Verify deployment URL
    const localhostUrl = await page.locator('[data-testid=\"localhost-url\"]').textContent();
    expect(localhostUrl).toContain('http://localhost:');
    
    // Navigate to deployed app
    const appPage = await page.context().newPage();
    await appPage.goto(localhostUrl);
    
    // Verify app is working
    await expect(appPage.locator('body')).toContainText('Grocery App');
  });
  
  test('should handle agent errors gracefully', async ({ page }) => {
    await page.goto('http://localhost:3000/mission-control');
    
    // Create team with invalid configuration
    await page.click('[data-testid=\"create-team-button\"]');
    await page.fill('[data-testid=\"project-name\"]', '');
    await page.click('[data-testid=\"submit-button\"]');
    
    // Should show validation error
    await expect(page.locator('[data-testid=\"validation-error\"]')).toBeVisible();
    
    // Create valid team
    await page.fill('[data-testid=\"project-name\"]', 'Test App');
    await page.click('[data-testid=\"submit-button\"]');
    
    // Simulate agent failure
    await page.evaluate(() => {
      window.__TEST_SIMULATE_AGENT_FAILURE__ = true;
    });
    
    // Wait for error
    await page.waitForSelector('[data-testid=\"agent-error\"]', { timeout: 30000 });
    
    // Verify error message
    const errorMessage = await page.locator('[data-testid=\"agent-error-message\"]').textContent();
    expect(errorMessage).toContain('Failed to');
    
    // Should allow retry
    await page.click('[data-testid=\"retry-button\"]');
    
    // Agent should recover
    await expect(page.locator('[data-testid=\"agent-status\"]')).toHaveText('Working');
  });
});
```

### 12.5 Performance Testing

```rust
// tests/performance/benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_agent_spinup(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function(\"agent_spinup\", |b| {
        b.to_async(&rt).iter(|| async {
            let orchestrator = AgentOrchestrator::new().await.unwrap();
            let _agent = orchestrator.create_agent(AgentRole::Planner).await.unwrap();
        })
    });
}

fn benchmark_model_generation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let router = rt.block_on(ModelRouter::new()).unwrap();
    
    c.bench_function(\"model_generation_local\", |b| {
        b.to_async(&rt).iter(|| async {
            let _response = router.generate(\"Write a function\", ModelPreference::Fast).await.unwrap();
        })
    });
    
    c.bench_function(\"model_generation_cloud\", |b| {
        b.to_async(&rt).iter(|| async {
            let _response = router.generate(\"Design architecture\", ModelPreference::Reasoning).await.unwrap();
        })
    });
}

fn benchmark_file_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let workspace = rt.block_on(create_test_workspace()).unwrap();
    
    c.bench_function(\"file_read_1mb\", |b| {
        b.to_async(&rt).iter(|| async {
            let _content = workspace.read_file(\"test_1mb.txt\").await.unwrap();
        })
    });
    
    c.bench_function(\"file_write_1mb\", |b| {
        b.to_async(&rt).iter(|| async {
            let content = vec![0u8; 1_000_000];
            workspace.write_file(\"test_write.txt\", &content).await.unwrap();
        })
    });
}

criterion_group!(
    benches,
    benchmark_agent_spinup,
    benchmark_model_generation,
    benchmark_file_operations
);
criterion_main!(benches);
```

---

## 13. DEPLOYMENT ARCHITECTURE

### 13.1 Development Environment

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  jag-ide-backend:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - \"8080:8080\"
    environment:
      - RUST_LOG=debug
      - DATABASE_URL=sqlite://./data/jag.db
      - REDIS_URL=redis://redis:6379
      - OLLAMA_URL=http://ollama:11434
    volumes:
      - ./data:/app/data
      - ./workspaces:/app/workspaces
    depends_on:
      - redis
      - ollama
    
  jag-ide-frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile.dev
    ports:
      - \"3000:3000\"
    environment:
      - REACT_APP_API_URL=http://localhost:8080
    volumes:
      - ./frontend/src:/app/src
    depends_on:
      - jag-ide-backend
    
  redis:
    image: redis:7-alpine
    ports:
      - \"6379:6379\"
    volumes:
      - redis_data:/data
    
  ollama:
    image: ollama/ollama:latest
    ports:
      - \"11434:11434\"
    volumes:
      - ollama_models:/root/.ollama
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
    
  postgres:
    image: postgres:15-alpine
    ports:
      - \"5432:5432\"
    environment:
      - POSTGRES_USER=jag
      - POSTGRES_PASSWORD=jag
      - POSTGRES_DB=jag
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  redis_data:
  ollama_models:
  postgres_data:
```

### 13.2 Production Deployment

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  jag-ide:
    image: jagide/jag-ide:latest
    ports:
      - \"80:80\"
      - \"443:443\"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgres://user:pass@postgres:5432/jag
      - REDIS_URL=redis://redis:6379
      - OLLAMA_URL=http://ollama:11434
      - JWT_SECRET=${JWT_SECRET}
    volumes:
      - jag_workspaces:/workspaces
      - jag_models:/models
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '4'
          memory: 8G
        reservations:
          cpus: '2'
          memory: 4G
    healthcheck:
      test: [\"CMD\", \"curl\", \"-f\", \"http://localhost:8080/health\"]
      interval: 30s
      timeout: 10s
      retries: 3
    
  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=${DB_USER}
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=jag
    volumes:
      - postgres_data:/var/lib/postgresql/data
    deploy:
      resources:
        limits:
          memory: 4G
    
  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    deploy:
      resources:
        limits:
          memory: 2G
    
  ollama:
    image: ollama/ollama:latest
    volumes:
      - ollama_models:/root/.ollama
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]

volumes:
  jag_workspaces:
  jag_models:
  postgres_data:
  redis_data:
  ollama_models:
```

### 13.3 Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: jag-ide
  namespace: jag-ide
spec:
  replicas: 3
  selector:
    matchLabels:
      app: jag-ide
  template:
    metadata:
      labels:
        app: jag-ide
    spec:
      containers:
      - name: jag-ide
        image: jagide/jag-ide:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: jag-secrets
              key: database-url
        - name: REDIS_URL
          value: \"redis://redis-service:6379\"
        - name: OLLAMA_URL
          value: \"http://ollama-service:11434\"
        resources:
          requests:
            memory: \"2Gi\"
            cpu: \"1000m\"
          limits:
            memory: \"4Gi\"
            cpu: \"2000m\"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: jag-ide-service
  namespace: jag-ide
spec:
  selector:
    app: jag-ide
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: jag-ide-ingress
  namespace: jag-ide
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - jag-ide.example.com
    secretName: jag-ide-tls
  rules:
  - host: jag-ide.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: jag-ide-service
            port:
              number: 80
```

---

## 14. INTEGRATION POINTS

### 14.1 VS Code Extension Compatibility

```typescript
// src/vscode-compat/extension-host.ts
import * as vscode from 'vscode';

export class VSCodeExtensionHost {
  private extensionContext: vscode.ExtensionContext;
  private api: typeof vscode;
  
  constructor(context: vscode.ExtensionContext) {
    this.extensionContext = context;
    this.api = vscode;
  }
  
  public activateExtension(extensionId: string): Promise<void> {
    const extension = this.api.extensions.getExtension(extensionId);
    if (!extension) {
      throw new Error(`Extension ${extensionId} not found`);
    }
    
    return extension.activate();
  }
  
  public registerCommand(command: string, callback: (...args: any[]) => any): vscode.Disposable {
    return this.api.commands.registerCommand(command, callback);
  }
  
  public createTextEditor(document: vscode.TextDocument): vscode.TextEditor {
    return this.api.window.createTextEditor(document);
  }
  
  public getActiveTextEditor(): vscode.TextEditor | undefined {
    return this.api.window.activeTextEditor;
  }
}
```

### 14.2 LSP Integration

```rust
// src/lsp/server.rs
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct JagLanguageServer {
    client: Client,
    backend: Backend,
}

impl LanguageServer for JagLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![\".\".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }
    
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, \"Jag LSP server initialized!\")
            .await;
    }
    
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let items = self.backend.get_completions(params).await;
        Ok(Some(CompletionResponse::Array(items)))
    }
    
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        Ok(self.backend.get_hover(params).await)
    }
    
    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        Ok(self.backend.goto_definition(params).await)
    }
}
```

### 14.3 Git Integration

```rust
// src/git/integration.rs
use git2::{Repository, BranchType, StatusOptions};

pub struct GitIntegration {
    repo: Repository,
}

impl GitIntegration {
    pub fn open(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }
    
    pub fn get_status(&self) -> Result<Vec<StatusEntry>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        
        let statuses = self.repo.statuses(Some(&mut opts))?;
        
        let entries: Vec<StatusEntry> = statuses
            .iter()
            .map(|entry| StatusEntry {
                path: entry.path().unwrap().to_string(),
                status: self.parse_status(entry.status()),
            })
            .collect();
        
        Ok(entries)
    }
    
    pub fn stage_changes(&self, paths: &[&str]) -> Result<()> {
        let mut index = self.repo.index()?;
        
        for path in paths {
            index.add_path(path.as_ref())?;
        }
        
        index.write()?;
        
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        // Update HEAD
        let head = self.repo.head()?;
        let parent = head.peel_to_commit()?;
        
        Ok(())
    }
    
    pub fn commit(&self, message: &str) -> Result<String> {
        let signature = self.repo.signature()?;
        let head = self.repo.head()?;
        let parent = head.peel_to_commit()?;
        
        let tree = {
            let mut index = self.repo.index()?;
            let tree_id = index.write_tree()?;
            self.repo.find_tree(tree_id)?
        };
        
        let commit_oid = self.repo.commit(
            Some(\"HEAD\"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent],
        )?;
        
        Ok(commit_oid.to_string())
    }
    
    pub fn create_branch(&self, name: &str, target: &str) -> Result<()> {
        let head = self.repo.head()?;
        let target_commit = self.repo.revparse_single(target)?.peel_to_commit()?;
        
        self.repo.branch(name, &target_commit, false)?;
        
        Ok(())
    }
    
    pub fn merge_branch(&self, branch_name: &str) -> Result<MergeResult> {
        let branch = self.repo.find_branch(branch_name, BranchType::Local)?;
        let branch_commit = branch.get().peel_to_commit()?;
        
        let mut checkout_opts = git2::build::CheckoutBuilder::default();
        let mut merge_opts = git2::MergeOptions::new();
        
        match self.repo.merge_commits(&self.repo.head()?.peel_to_commit()?, &branch_commit, Some(&mut merge_opts)) {
            Ok(index) => {
                if index.has_conflicts() {
                    Ok(MergeResult::Conflict)
                } else {
                    // Auto-merge successful
                    Ok(MergeResult::Success)
                }
            }
            Err(_) => Ok(MergeResult::Conflict),
        }
    }
}

pub enum MergeResult {
    Success,
    Conflict,
}
```

---

## 15. TECHNICAL CONSTRAINTS & ASSUMPTIONS

### 15.1 Constraints

1. **Platform Support:**
   - Must support Windows 10+, macOS 11+, Ubuntu 20.04+
   - ARM64 support required for Apple Silicon
   - GPU acceleration requires OpenGL 4.5+ or Vulkan 1.2+

2. **Performance:**
   - Maximum memory usage: 2GB (active), 500MB (idle)
   - Startup time must be <2 seconds on SSD
   - Model response latency <5s (p95) for local models

3. **Security:**
   - All file operations must be sandboxed
   - No network access without explicit permission
   - Secrets must be encrypted at rest
   - Audit logging required for all privileged operations

4. **Compatibility:**
   - Must maintain 90%+ VS Code extension API compatibility
   - Must support standard LSP and DAP protocols
   - Must work with existing Git repositories

5. **Resource Requirements:**
   - Minimum 8GB RAM
   - Minimum 4GB free disk space
   - GPU with 2GB VRAM recommended for AI features

### 15.2 Assumptions

1. **User Environment:**
   - Users have basic development environment (Node.js, Python, etc.)
   - Users have internet connectivity for cloud models
   - Users have sufficient disk space for models (10-50GB)

2. **Model Availability:**
   - Ollama will remain available and maintained
   - Cloud providers (Anthropic, OpenAI) will maintain stable APIs
   - Model licenses allow commercial use

3. **Infrastructure:**
   - Redis and SQLite will meet performance requirements
   - Network latency to cloud providers <200ms
   - GPU drivers are properly installed

4. **Development:**
   - VS Code OSS will remain open-source
   - Rust ecosystem will continue to mature
   - Community will contribute extensions

5. **Legal:**
   - VS Code fork complies with MIT license
   - Model usage complies with terms of service
   - User data handling complies with GDPR/CCPA

---

## 16. RISK ASSESSMENT & MITIGATION

### 16.1 Technical Risks

| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| **Performance degradation with large codebases** | Medium | High | Implement lazy loading, indexing, caching; benchmark regularly |
| **Model inaccuracy or hallucinations** | High | Medium | Implement verification systems, human-in-the-loop approvals |
| **API instability in cloud models** | Medium | High | Implement fallback to local models, abstraction layers |
| **Security vulnerability in sandbox** | Low | Critical | Independent security audits, regular updates to kernel-level isolation |
| **User adoption friction for multi-agent system** | Medium | Medium | Intuitive UI/UX, progressive disclosure of complexity |
| **GPU compatibility issues** | Low | Medium | Software rendering fallback, cross-platform GPU abstraction |
| **Data loss in workspace operations** | Very Low | Critical | Automatic backups, Git-based versioning for artifacts |
| **Scalability of backend orchestration** | Medium | Medium | Distributed message bus, scalable task queue architecture |
| **Compliance issues (GDPR/CCPA)** | Low | High | Data minimization, explicit user consent, encrypted local storage |
| **Extension ecosystem incompatibility** | Medium | Medium | Continuous integration with popular extensions, community feedback |

---

## 17. APPENDICES

[To be completed with detailed API references, database schemas, and configuration guides]
