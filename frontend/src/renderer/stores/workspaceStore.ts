import { create } from 'zustand';
import { api } from '../services/api';

interface WorkspaceState {
  files: string[];
  activeFile: string | null;
  fileContent: string | null;
  isLoading: boolean;
  error: string | null;

  fetchFiles: () => Promise<void>;
  openFile: (path: string) => Promise<void>;
}

export const useWorkspaceStore = create<WorkspaceState>((set) => ({
  files: [],
  activeFile: null,
  fileContent: null,
  isLoading: false,
  error: null,

  fetchFiles: async () => {
    set({ isLoading: true });
    try {
      const files = await api.listFiles();
      set({ files, error: null });
    } catch (err: any) {
      set({ error: err.message });
    } finally {
      set({ isLoading: false });
    }
  },

  openFile: async (path: string) => {
    set({ isLoading: true, activeFile: path });
    try {
      const content = await api.readFile(path);
      set({ fileContent: content, error: null });
    } catch (err: any) {
      set({ error: err.message, fileContent: null });
    } finally {
      set({ isLoading: false });
    }
  },
}));
