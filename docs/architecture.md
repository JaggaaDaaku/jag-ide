# Jag IDE Architecture

## Overview
Jag IDE is an agent-first autonomous development platform designed to build full-stack applications with minimal human intervention. It leverages a multi-agent orchestration layer, a hardened sandbox environment, and a distributed LLM routing system.

## System Components

### 1. Multi-Agent Orchestration (`jag-agents`)
A collection of specialized agents that collaborate on software tasks:
- **Planner Agent**: Generates PRDs, architecture diagrams, and API specs.
- **Backend Agent**: Implements business logic, data models, and migrations.
- **Frontend Agent**: Builds React components, styles, and page layouts.
- **Integration Agent**: Wires components together and generates test suites.
- **Git Agent**: Manages version control and pull request lifecycles.

### 2. Workflow Engine (`jag-workflow`)
The brain of the system. It breaks down high-level missions into a directed acyclic graph (DAG) of tasks, handles dependencies, and routes tasks to the appropriate agents.

### 3. Hardened Sandbox (`jag-sandbox`)
A secure execution environment for running generated code, tests, and external commands. It features:
- Path traversal protection.
- Command denylisting.
- Resource limits (CPU, memory).
- Cryptographically signed audit logs.

### 4. Model Routing (`jag-models`)
A provider-agnostic bridge to LLMs (OpenAI, Anthropic, Ollama). It handles model selection based on task type (Code Generation vs. Reasoning) and tracks performance benchmarks.

### 5. Unified Database (`jag-db`)
A SQLite-based persistence layer for storing:
- Workspace state.
- Agent history.
- Task DAGs.
- Artifact metadata.
- Audit logs.

## Data Flow
1. **Mission Input**: User provides a project description.
2. **Planning**: Planner agent generates the blueprint (PRD, Spec).
3. **Execution**: Backend and Frontend agents work in parallel on implementation tasks.
4. **Integration**: Integration agent wires the pieces and validates with tests.
5. **Deployment**: Git agent commits the code and prepares for release.
