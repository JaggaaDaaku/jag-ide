import { app, BrowserWindow, ipcMain, shell } from 'electron';
import * as path from 'path';
import { spawn, ChildProcess } from 'child_process';

let mainWindow: BrowserWindow | null = null;
let backendProcess: ChildProcess | null = null;

function spawnBackend(): void {
  const isDev = !app.isPackaged;
  
  // Path to the backend binary
  let binPath: string;
  if (isDev) {
    binPath = path.join(__dirname, '..', '..', 'target', 'debug', 'jag-server.exe');
  } else {
    binPath = path.join(process.resourcesPath, 'bin', 'jag-server.exe');
  }

  console.log(`Launching backend from: ${binPath}`);

  backendProcess = spawn(binPath, [], {
    env: {
      ...process.env,
      JAG_DATABASE_URL: `sqlite://${path.join(app.getPath('userData'), 'jag.db')}`,
      JAG_OLLAMA_BASE_URL: 'http://localhost:11434',
    },
    stdio: 'inherit', // Relay logs to Electron's stdout
  });

  backendProcess.on('error', (err) => {
    console.error('Failed to start backend process:', err);
  });

  backendProcess.on('exit', (code) => {
    console.log(`Backend process exited with code ${code}`);
  });
}

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1400,
    height: 900,
    minWidth: 900,
    minHeight: 600,
    frame: false,            // Zero-chrome: custom titlebar
    titleBarStyle: 'hidden',
    backgroundColor: '#1e1e2e',
    show: false,             // Show only once ready-to-show
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
    },
  });

  mainWindow.loadFile(path.join(__dirname, '..', 'src', 'renderer', 'index.html'));

  mainWindow.once('ready-to-show', () => {
    mainWindow?.show();
    mainWindow?.focus();
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

ipcMain.on('window:minimize', () => mainWindow?.minimize());
ipcMain.on('window:maximize', () => {
  if (mainWindow?.isMaximized()) mainWindow.unmaximize();
  else mainWindow?.maximize();
});
ipcMain.on('window:close', () => mainWindow?.close());

ipcMain.on('shell:openExternal', (_event, url: string) => {
  shell.openExternal(url);
});

app.whenReady().then(() => {
  spawnBackend();
  createWindow();
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (backendProcess) {
    backendProcess.kill();
  }
  if (process.platform !== 'darwin') app.quit();
});

app.on('will-quit', () => {
  if (backendProcess) {
    backendProcess.kill();
  }
});
