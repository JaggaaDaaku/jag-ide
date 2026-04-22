import { create } from 'zustand';
import { api, DashboardResponse, AgentStateDto, ArtifactMetadataDto, WorkflowStatusDto } from '../services/api';

interface AgentState {
  agents: AgentStateDto[];
  workflow: WorkflowStatusDto | null;
  recentArtifacts: ArtifactMetadataDto[];
  availableModels: string[];
  isPolling: boolean;
  error: string | null;

  startPolling: () => void;
  stopPolling: () => void;
  fetchDashboard: () => Promise<void>;
  startWorkflow: (description: string) => Promise<void>;
  submitTask: (role: string, taskType: string, payload: any) => Promise<void>;
  approveArtifact: (id: string) => Promise<void>;
  rejectArtifact: (id: string) => Promise<void>;
}

let pollInterval: NodeJS.Timeout | null = null;

export const useAgentStore = create<AgentState>((set, get) => ({
  agents: [],
  workflow: null,
  recentArtifacts: [],
  availableModels: [],
  isPolling: false,
  error: null,

  fetchDashboard: async () => {
    try {
      const data = await api.fetchDashboard();
      set({ 
        agents: data.agents, 
        workflow: data.workflow, 
        recentArtifacts: data.recent_artifacts,
        availableModels: data.available_models,
        error: null 
      });
    } catch (err: any) {
      set({ error: err.message });
      console.error('Dashboard poll failed:', err);
    }
  },

  startPolling: () => {
    if (pollInterval) return;
    set({ isPolling: true });
    get().fetchDashboard();
    pollInterval = setInterval(() => {
      get().fetchDashboard();
    }, 3000);
  },

  stopPolling: () => {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
    set({ isPolling: false });
  },

  startWorkflow: async (description: string) => {
    try {
      await api.startWorkflow(description);
      get().fetchDashboard();
    } catch (err: any) {
      set({ error: err.message });
    }
  },

  submitTask: async (role, taskType, payload) => {
    try {
      await api.submitTask(role, taskType, payload);
      get().fetchDashboard();
    } catch (err: any) {
      set({ error: err.message });
    }
  },

  approveArtifact: async (id) => {
    try {
      await api.approveArtifact(id);
      get().fetchDashboard();
    } catch (err: any) {
      set({ error: err.message });
    }
  },

  rejectArtifact: async (id) => {
    try {
      await api.rejectArtifact(id);
      get().fetchDashboard();
    } catch (err: any) {
      set({ error: err.message });
    }
  },
}));
