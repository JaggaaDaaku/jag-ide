# API Reference

## Authentication
Jag IDE uses JWT-based authentication. In development mode (`OIDC_ENABLED=false`), a mock admin user is automatically logged in.

## Workspace API

### `GET /api/workspaces`
List all active workspaces.

### `POST /api/workspaces`
Create a new workspace.
**Body:** `{ "name": "Project Name", "root_path": "/path/to/project" }`

## Mission Control API

### `POST /api/missions/launch`
Launch a new autonomous development mission.
**Body:** `{ "description": "Build a task manager", "tech_stack": "Next.js + Tailwind" }`

### `GET /api/missions/:id/status`
Track mission progress and agent activity.

## Artifact API

### `GET /api/artifacts/recent`
Get the latest generated code, diagrams, and documents.

### `GET /api/artifacts/:id/download`
Retrieve raw artifact content.

## Model Benchmarks API

### `GET /api/benchmarks`
Get performance metrics for configured LLM models (TPS, Latency).

### `POST /api/benchmarks/run`
Trigger a real-time benchmark session.
**Body:** `{ "model_id": "llama3", "test_type": "code-gen" }`
