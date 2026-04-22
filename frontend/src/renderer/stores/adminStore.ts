import { create } from 'zustand';
import { api, DailyUsageStat, AuditEntry } from '../services/api';

interface AdminState {
  analytics: DailyUsageStat[];
  auditLogs: AuditEntry[];
  approvalQueue: any[];
  workflowHistory: any[];
  isLoading: boolean;
  error: string | null;

  fetchAnalytics: (days?: number) => Promise<void>;
  fetchAuditLogs: (page?: number, limit?: number) => Promise<void>;
  verifyAudit: (id: number) => Promise<boolean>;
  fetchApprovals: () => Promise<void>;
  decideApproval: (id: string, status: string, comments?: string) => Promise<void>;
  fetchWorkflowHistory: () => Promise<void>;
}

export const useAdminStore = create<AdminState>((set, get) => ({
  analytics: [],
  auditLogs: [],
  approvalQueue: [],
  workflowHistory: [],
  isLoading: false,
  error: null,

  fetchAnalytics: async (days = 30) => {
    set({ isLoading: true, error: null });
    try {
      const data = await api.fetchAnalytics(days);
      set({ analytics: data, isLoading: false });
    } catch (err: any) {
      set({ error: err.message, isLoading: false });
    }
  },

  fetchAuditLogs: async (page = 0, limit = 50) => {
    set({ isLoading: true, error: null });
    try {
      const data = await api.fetchAuditLogs(page, limit);
      set({ auditLogs: data, isLoading: false });
    } catch (err: any) {
      set({ error: err.message, isLoading: false });
    }
  },

  verifyAudit: async (id: number) => {
    try {
      const res = await api.verifyAuditEntry(id);
      return res.verified;
    } catch (err) {
      console.error('Audit verification failed:', err);
      return false;
    }
  },

  fetchApprovals: async () => {
    set({ isLoading: true, error: null });
    try {
      const data = await api.fetchApprovals();
      set({ approvalQueue: data, isLoading: false });
    } catch (err: any) {
      set({ error: err.message, isLoading: false });
    }
  },

  decideApproval: async (id: string, status: string, comments?: string) => {
    try {
      await api.submitApprovalDecision(id, { status, comments });
      await get().fetchApprovals();
    } catch (err: any) {
      set({ error: err.message });
    }
  },

  fetchWorkflowHistory: async () => {
    set({ isLoading: true, error: null });
    try {
      const data = await api.fetchWorkflowHistory();
      set({ workflowHistory: data, isLoading: false });
    } catch (err: any) {
      set({ error: err.message, isLoading: false });
    }
  },
}));
