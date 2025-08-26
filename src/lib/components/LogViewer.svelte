<script>
  import { open } from '@tauri-apps/plugin-shell';
  import { appLogDir } from '@tauri-apps/api/path';

  async function openLogFolder() {
    try {
      const logDir = await appLogDir();
      await open(logDir);
    } catch (err) {
      console.error('Failed to open log folder:', err);
      alert('Failed to open log folder: ' + err.message);
    }
  }

  async function showLogPath() {
    try {
      const logDir = await appLogDir();
      const logFile = `${logDir}/queryowl.log`;
      alert(`Log file location:\n${logFile}\n\nClick "Open Log Folder" to access it.`);
    } catch (err) {
      console.error('Failed to get log path:', err);
      alert('Failed to get log path: ' + err.message);
    }
  }
</script>

<div class="fixed bottom-4 right-4 z-50">
  <div class="flex flex-col space-y-2">
    <button
      onclick={openLogFolder}
      class="bg-gray-600 hover:bg-gray-700 text-white text-xs px-3 py-1 rounded shadow-lg transition-colors"
      title="Open log folder"
    >
      📁 Logs
    </button>
    <button
      onclick={showLogPath}
      class="bg-gray-500 hover:bg-gray-600 text-white text-xs px-3 py-1 rounded shadow-lg transition-colors"
      title="Show log file path"
    >
      📍 Path
    </button>
  </div>
</div>