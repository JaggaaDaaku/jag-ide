import { contextBridge, ipcRenderer } from 'electron';

/**
 * Exposes a safe, typed API to the renderer process via `window.jagBridge`.
 * No direct Node.js access is allowed in the renderer (contextIsolation: true).
 */
contextBridge.exposeInMainWorld('jagBridge', {
  window: {
    minimize: () => ipcRenderer.send('window:minimize'),
    maximize: () => ipcRenderer.send('window:maximize'),
    close:    () => ipcRenderer.send('window:close'),
    openFolder: () => ipcRenderer.invoke('window:openFolder'),
    newFile: () => ipcRenderer.invoke('window:newFile'),
  },

  // Shell utilities
  shell: {
    openExternal: (url: string) => ipcRenderer.send('shell:openExternal', url),
  },

  // Event subscription helpers
  on:  (channel: string, callback: (...args: unknown[]) => void) =>
    ipcRenderer.on(channel, (_event, ...args) => callback(...args)),
  off: (channel: string, callback: (...args: unknown[]) => void) =>
    ipcRenderer.removeListener(channel, callback),
  once: (channel: string, callback: (...args: unknown[]) => void) =>
    ipcRenderer.once(channel, (_event, ...args) => callback(...args)),
});

// Type declaration — imported by renderer via window.jagBridge
export type JagBridge = typeof import('./preload');
