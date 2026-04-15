/**
 * types.ts — Jag IDE TypeScript type definitions
 *
 * AUTO-GENERATED mirror of crates/jag-core/src/types.rs
 * Mapping rules:
 *   snake_case       → camelCase
 *   Option<T>        → T | undefined  (optional field with ?)
 *   Vec<T>           → T[]
 *   HashMap<K,V>     → Record<K, V>
 *   serde_json::Value→ unknown
 *   DateTime<Utc>    → string  (ISO 8601)
 *   Vec<u8>          → string  (base64)
 *   UUID newtypes    → branded string
 *   u8 / usize       → number
 */

// ---------------------------------------------------------------------------
// Branded ID types (UUID-backed, distinct at the type level)
// ---------------------------------------------------------------------------

/** Opaque wrapper — underlying value is a UUID string. */
declare const __brand: unique symbol;
type Brand<T, B> = T & { readonly [__brand]: B };

export type AgentId     = Brand<string, 'AgentId'>;
export type TeamId      = Brand<string, 'TeamId'>;
export type TaskId      = Brand<string, 'TaskId'>;
export type ArtifactId  = Brand<string, 'ArtifactId'>;
export type WorkspaceId = Brand<string, 'WorkspaceId'>;
export type ProjectId   = Brand<string, 'ProjectId'>;
export type MessageId   = Brand<string, 'MessageId'>;

/** ModelId is a plain string (model name, e.g. "qwen2.5-coder:14b"). */
export type ModelId    = string;
/** ProviderId is a plain string (e.g. "anthropic"). */
export type ProviderId = string;

// ---------------------------------------------------------------------------
// Helper to cast a raw string to a branded ID (use at API boundaries only)
// ---------------------------------------------------------------------------

export const asAgentId     = (s: string): AgentId     => s as AgentId;
export const asTeamId      = (s: string): TeamId      => s as TeamId;
export const asTaskId      = (s: string): TaskId      => s as TaskId;
export const asArtifactId  = (s: string): ArtifactId  => s as ArtifactId;
export const asWorkspaceId = (s: string): WorkspaceId => s as WorkspaceId;
export const asProjectId   = (s: string): ProjectId   => s as ProjectId;
export const asMessageId   = (s: string): MessageId   => s as MessageId;

// ---------------------------------------------------------------------------
// Enums — plain variants (string literal unions)
// ---------------------------------------------------------------------------

/** Mirrors: enum AgentRole */
export type AgentRole =
  | 'Planner'
  | 'Backend'
  | 'Frontend'
  | 'Integration';

/** Mirrors: enum AgentStatus */
export type AgentStatus =
  | 'Idle'
  | 'Working'
  | 'Completed'
  | 'Error';

/** Mirrors: enum TaskStatus */
export type TaskStatus =
  | 'Pending'
  | 'Running'
  | 'Completed'
  | 'Failed'
  | 'Cancelled';

/** Mirrors: enum TaskType */
export type TaskType =
  | 'GeneratePRD'
  | 'DesignArchitecture'
  | 'DefineDataModels'
  | 'SpecifyAPIs'
  | 'ImplementAPI'
  | 'GenerateModels'
  | 'ImplementAuth'
  | 'BuildUI'
  | 'GenerateComponents'
  | 'GenerateStyles'
  | 'IntegrateAPI'
  | 'Integrate'
  | 'RunTests'
  | 'Deploy'
  | 'GenerateReadme';

/** Mirrors: enum ArtifactType */
export type ArtifactType =
  | 'PRD'
  | 'ArchitectureDiagram'
  | 'APISpecification'
  | 'DatabaseSchema'
  | 'BackendCode'
  | 'FrontendCode'
  | 'TestReport'
  | 'DeploymentPackage'
  | 'CodeDiff';

/** Mirrors: enum SecurityTier */
export type SecurityTier = 'Off' | 'Auto' | 'Turbo';

/** Mirrors: enum VerificationStatus */
export type VerificationStatus = 'Pending' | 'Approved' | 'Rejected';

/** Mirrors: enum Priority */
export type Priority = 'Low' | 'Normal' | 'High' | 'Critical';

/** Mirrors: enum DependencyType */
export type DependencyType = 'Hard' | 'Soft' | 'Parallel';

/** Mirrors: enum ModelPreference */
export type ModelPreference = 'Reasoning' | 'CodeGeneration' | 'Fast';

// ---------------------------------------------------------------------------
// Enums with associated data → discriminated unions
// ---------------------------------------------------------------------------

/**
 * Mirrors: enum ModelProvider { Local, Cloud(String) }
 * Serde serializes as:
 *   Local      → "Local"
 *   Cloud("x") → { "Cloud": "x" }
 */
export type ModelProvider =
  | 'Local'
  | { Cloud: string };

/**
 * Mirrors: enum MessageType { ArtifactReady(ArtifactType), TaskRequest(TaskType), ... }
 * Serde serializes enum-with-data as { "VariantName": payload }.
 */
export type MessageType =
  | { ArtifactReady: ArtifactType }
  | { TaskRequest: TaskType }
  | { StatusUpdate: AgentStatus }
  | { DependencyResolved: TaskId }
  | { ErrorReport: string };

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/**
 * Mirrors: struct MessagePayload
 * HashMap<String, String> → Record<string, string>
 * serde_json::Value       → unknown
 */
export interface MessagePayload {
  /** Option<ArtifactId> */
  artifactId?: ArtifactId;
  /** Option<TaskId> */
  taskId?: TaskId;
  /** serde_json::Value — arbitrary JSON payload */
  data: unknown;
  /** HashMap<String, String> */
  metadata: Record<string, string>;
}

/**
 * Mirrors: struct AgentMessage
 * `from` is a reserved word in JS/TS — kept as-is since it's valid as an object key.
 */
export interface AgentMessage {
  id: MessageId;
  /** `from` field (AgentId of sender) */
  from: AgentId;
  /** Option<AgentId> — None means broadcast */
  to?: AgentId;
  /** DateTime<Utc> as ISO 8601 string */
  timestamp: string;
  messageType: MessageType;
  payload: MessagePayload;
  priority: Priority;
  /** Option<MessageId> */
  correlationId?: MessageId;
}

/**
 * Mirrors: struct Task
 */
export interface Task {
  id: TaskId;
  /** Option<AgentId> */
  agentId?: AgentId;
  taskType: TaskType;
  status: TaskStatus;
  priority: Priority;
  /** serde_json::Value — task-specific input data */
  payload: unknown;
  /** Vec<TaskId> */
  dependencies: TaskId[];
}

/**
 * Mirrors: struct ArtifactMetadata
 * usize → number
 */
export interface ArtifactMetadata {
  createdBy: AgentId;
  /** DateTime<Utc> as ISO 8601 string */
  timestamp: string;
  version: string;
  format: string;
  /** usize — byte size of artifact content */
  size: number;
}

/**
 * Mirrors: struct Artifact
 * Vec<u8> → string (base64-encoded)
 */
export interface Artifact {
  id: ArtifactId;
  artifactType: ArtifactType;
  /** Vec<u8> serialized as base64 string */
  content: string;
  metadata: ArtifactMetadata;
  verificationStatus: VerificationStatus;
}

/**
 * Mirrors: struct Workspace
 */
export interface Workspace {
  id: WorkspaceId;
  name: string;
  rootPath: string;
  /** DateTime<Utc> as ISO 8601 string */
  createdAt: string;
  /** DateTime<Utc> as ISO 8601 string */
  modifiedAt: string;
  /** Vec<AgentId> */
  agents: AgentId[];
  /** Vec<ArtifactId> */
  artifacts: ArtifactId[];
}

/**
 * Mirrors: struct AgentState
 */
export interface AgentState {
  status: AgentStatus;
  /** Option<TaskId> */
  currentTask?: TaskId;
  /** u8 — progress percentage 0–100 */
  progress: number;
  /** DateTime<Utc> as ISO 8601 string */
  lastHeartbeat: string;
}
