<script>
  import { invoke } from '@tauri-apps/api/core';
  import { appLogDir } from '@tauri-apps/api/path';
  import { info, error } from '@tauri-apps/plugin-log';

  async function openLogFolder() {
    try {
      info('LogViewer: Attempting to open log folder...');
      await invoke('open_log_folder');
      info('LogViewer: Successfully opened log folder');
    } catch (err) {
      error(`LogViewer: Failed to open log folder - ${err.message || err}`);
      console.error('Failed to open log folder:', err);
      alert('Failed to open log folder: ' + (err.message || err));
    }
  }

  let showPath = $state(false);
  let logPath = $state('');

  async function showLogPath() {
    try {
      info('LogViewer: Getting log path...');
      logPath = await invoke('get_log_path');
      info(`LogViewer: Log file path is: ${logPath}`);
      showPath = true;
      info(`LogViewer: showPath state set to: ${showPath}`);
      console.log('Debug: showPath =', showPath, 'logPath =', logPath);
      // Auto-hide after 5 seconds
      setTimeout(() => { 
        showPath = false;
        info('LogViewer: Auto-hiding path popup');
      }, 5000);
    } catch (err) {
      error(`LogViewer: Failed to get log path - ${err.message || err}`);
      console.error('Failed to get log path:', err);
    }
  }
</script>

<div class="fixed bottom-4 right-4 z-50">
  <div class="flex flex-col space-y-2">
    <button
      onclick={openLogFolder}
      class="bg-secondary hover:bg-secondary/80 text-secondary-foreground text-xs px-3 py-1 rounded shadow-lg transition-colors"
      title="Open log folder"
    >
      📁 Logs
    </button>
    <button
      onclick={showLogPath}
      class="bg-surface-200-700 hover:bg-surface-200-700/80 text-surface-500 text-xs px-3 py-1 rounded shadow-lg transition-colors"
      title="Show log file path"
    >
      📍 Path
    </button>
    
{#if showPath}
      <div class="card border border-border rounded-lg shadow-xl p-4 max-w-sm backdrop-blur-sm">
        <div class="flex items-center justify-between mb-2">
          <h3 class="text-sm font-semibold text-card-foreground flex items-center gap-1">
            📋 Log File Path
          </h3>
          <button 
            onclick={() => showPath = false}
            class="text-surface-500 hover:text-card-foreground rounded p-1 hover:bg-accent transition-colors"
            aria-label="Close"
          >
            ✕
          </button>
        </div>
        <div class="bg-secondary/50 rounded p-2 mb-3">
          <p class="text-xs font-mono text-secondary-foreground break-all leading-relaxed">
            {logPath}
          </p>
        </div>
        <p class="text-xs text-surface-500">
          Click "📁 Logs" to open the folder containing this file.
        </p>
      </div>
    {/if}
  </div>
</div>