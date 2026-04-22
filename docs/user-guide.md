# User Guide

## Getting Started

### Installation
1. Download the latest installer from the [Releases](https://github.com/JaggaaDaaku/jag-ide/releases) page.
2. Run `Jag-IDE-Setup.exe` and follow the instructions.
3. Ensure [Ollama](https://ollama.com) is installed and running if you plan to use local models.

### Initial Setup
On first launch, Jag IDE will initialize its local database and default workspace directory (`~/jag-workspaces`).

## Starting Your First Mission

1. **Describe Your App**: Open the "New Mission" tab and describe what you want to build in plain English.
2. **Select Tech Stack**: (Optional) Specify your preferred frameworks.
3. **Launch**: Click the "Launch" button to start the autonomous loop.

## Monitoring Progress

The **Mission Control** dashboard shows real-time activity:
- **Planner Agent**: Watch as it drafts your PRD and Architecture.
- **Backend/Frontend Agents**: Monitor code generation progress.
- **Live Logs**: View the unified output from all active agents.

## Reviewing Artifacts

Generated code and documents appear in the **Artifacts** tab. You can:
- View PRDs and Diagrams.
- Download generated code files.
- Approve or Reject proposed changes.

## Security Tiers

You can configure the agent's autonomy level in Settings:
- **Auto (Default)**: Agents can read files and run tests but need permission for writes or deletions.
- **Turbo**: Fully autonomous mode. Recommended for internal use only.
- **Off**: Agents only suggest code but cannot execute anything.
