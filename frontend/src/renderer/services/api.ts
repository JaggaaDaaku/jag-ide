const BASE_URL = 'http://127.0.0.1:8080/api';

export interface PullRequestDto {
  number: number;
  html_url: string;
  state: string;
}

export interface DashboardResponse {
  agents: AgentStateDto[];
  workflow: WorkflowStatusDto;
  recent_artifacts: ArtifactMetadataDto[];
  available_models: string[];
}

export interface AgentStateDto {
  role: string;
  status: string;
  progress: number;
  current_task: string | null;
}

export interface WorkflowStatusDto {
  is_complete: boolean;
  has_failures: boolean;
  status_counts: Record<string, number>;
}

export interface ArtifactMetadataDto {
  id: string;
  artifact_type: string;
  created_by: string;
  timestamp: string;
  version: string;
  format: string;
  size: number;
  verification_status: string;
}

export interface DailyUsageStat {
  date: string;
  calls: number;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  cost: number;
}

export interface AuditEntry {
  id: number;
  timestamp: string;
  user_id: string | null;
  agent_id: string | null;
  action: string;
  resource_type: string | null;
  resource_id: string | null;
  details: any;
  result: string;
  ip_address: string | null;
  signature: string | null;
}

export const api = {
  fetchAnalytics: async (days: number = 30): Promise<DailyUsageStat[]> => {
    const res = await fetch(`${BASE_URL}/admin/analytics?days=${days}`);
    if (!res.ok) throw new Error('Failed to fetch analytics (RBAC check failed or server error)');
    return res.json();
  },

  fetchAuditLogs: async (page: number = 0, limit: number = 50): Promise<AuditEntry[]> => {
    const res = await fetch(`${BASE_URL}/admin/audit?page=${page}&limit=${limit}`);
    if (!res.ok) throw new Error('Failed to fetch audit logs (RBAC check failed or server error)');
    return res.json();
  },

  verifyAuditEntry: async (id: number): Promise<{ verified: boolean; integrity: string }> => {
    const res = await fetch(`${BASE_URL}/admin/audit/${id}/verify`);
    if (!res.ok) throw new Error('Failed to verify audit entry');
    return res.json();
  },

  fetchDashboard: async (): Promise<DashboardResponse> => {
    const res = await fetch(`${BASE_URL}/dashboard`);
    if (!res.ok) throw new Error('Failed to fetch dashboard');
    return res.json();
  },

  startWorkflow: async (description: string): Promise<{ workflow_id: string; initial_task_id: string }> => {
    const res = await fetch(`${BASE_URL}/workflow/start`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ description }),
    });
    if (!res.ok) throw new Error('Failed to start workflow');
    return res.json();
  },

  submitTask: async (role: string, taskType: string, payload: any): Promise<{ task_id: string }> => {
    const res = await fetch(`${BASE_URL}/agents/${role}/task`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ task_type: taskType, payload }),
    });
    if (!res.ok) throw new Error('Failed to submit task');
    return res.json();
  },

  getArtifact: async (id: string): Promise<{ content_base64: string; type: string }> => {
    const res = await fetch(`${BASE_URL}/artifacts/${id}`);
    if (!res.ok) throw new Error('Failed to fetch artifact');
    return res.json();
  },

  approveArtifact: async (id: string): Promise<void> => {
    const res = await fetch(`${BASE_URL}/artifacts/${id}/approve`, { method: 'POST' });
    if (!res.ok) throw new Error('Failed to approve artifact');
  },

  rejectArtifact: async (id: string): Promise<void> => {
    const res = await fetch(`${BASE_URL}/artifacts/${id}/reject`, { method: 'POST' });
    if (!res.ok) throw new Error('Failed to reject artifact');
  },

  listModels: async (): Promise<{ models: string[] }> => {
    const res = await fetch(`${BASE_URL}/models`);
    if (!res.ok) throw new Error('Failed to fetch models');
    return res.json();
  },

  listFiles: async (): Promise<string[]> => {
    const res = await fetch(`${BASE_URL}/workspace/files`);
    if (!res.ok) throw new Error('Failed to fetch files');
    return res.json();
  },

  readFile: async (path: string): Promise<string> => {
    const res = await fetch(`${BASE_URL}/workspace/files/${path}`);
    if (!res.ok) throw new Error('Failed to read file');
    return res.text();
  },

  fetchApprovals: async (): Promise<any[]> => {
    const res = await fetch(`${BASE_URL}/admin/approvals`);
    if (!res.ok) throw new Error('Failed to fetch approvals');
    return res.json();
  },

  submitApprovalDecision: async (id: string, decision: { status: string; comments?: string }): Promise<void> => {
    const res = await fetch(`${BASE_URL}/admin/approvals/${id}/decide`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(decision),
    });
    if (!res.ok) throw new Error('Failed to submit approval decision');
  },

  fetchWorkflowHistory: async (): Promise<any[]> => {
    const res = await fetch(`${BASE_URL}/workflow/history`);
    if (!res.ok) throw new Error('Failed to fetch workflow history');
    return res.json();
  },

  fetchVisualResults: async (workflowId: string): Promise<any[]> => {
    const res = await fetch(`${BASE_URL}/workflow/${workflowId}/visual-results`);
    if (!res.ok) throw new Error('Failed to fetch visual results');
    return res.json();
  },

  listPullRequests: async (): Promise<PullRequestDto[]> => {
    const res = await fetch(`${BASE_URL}/git/prs`);
    if (!res.ok) throw new Error('Failed to fetch pull requests');
    return res.json();
  }
};
