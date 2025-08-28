<script>
  import { onMount } from 'svelte';
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { info, error, debug } from '@tauri-apps/plugin-log';

  let updateState = $state('');
  let showNotification = $state(false);
  let update = $state(null);
  let errorMessage = $state('');
  let autoHideTimer = $state(null);
  let downloadProgress = $state(0);

  onMount(() => {
    checkForUpdates();
  });

  async function checkForUpdates() {
    try {
      info('QueryOwl: Checking for updates...');
      update = await check();
      
      if (update?.available) {
        info(`QueryOwl: Update available - version ${update.version}`);
        updateState = 'available';
        showNotification = true;
      } else {
        info('QueryOwl: No updates available');
      }
    } catch (err) {
      error(`QueryOwl: Failed to check for updates - ${err.message || err}`);
      console.error('Failed to check for updates - Full error:', err);
      updateState = 'error';
      errorMessage = err.message || 'Failed to check for updates';
      showNotification = true;
      startAutoHide();
    }
  }

  async function installUpdate() {
    if (!update) return;

    try {
      info(`QueryOwl: Starting update installation to version ${update.version}`);
      debug(`QueryOwl: Update details - version: ${update.version}, current: ${update.currentVersion}`);
      
      updateState = 'installing';
      
      info('QueryOwl: Downloading and installing update...');
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          info(`QueryOwl: Download started, size: ${event.data.contentLength} bytes`);
        } else if (event.event === 'Progress') {
          downloadProgress = (event.data.chunkLength / event.data.contentLength) * 100;
          if (downloadProgress % 10 < 1) { // Log every 10%
            info(`QueryOwl: Download progress: ${downloadProgress.toFixed(0)}%`);
          }
        } else if (event.event === 'Finished') {
          info('QueryOwl: Download completed, installing...');
        }
      });
      
      info('QueryOwl: Update installed successfully');
      updateState = 'complete';
      startAutoHide();
      
      // Auto-restart after 3 seconds
      info('QueryOwl: Restarting application in 3 seconds...');
      setTimeout(() => {
        info('QueryOwl: Relaunching application...');
        relaunch();
      }, 3000);
    } catch (err) {
      error(`QueryOwl: Failed to install update - ${err.message || err}`);
      error(`QueryOwl: Error type: ${err?.constructor?.name}`);
      if (err?.stack) {
        debug(`QueryOwl: Error stack: ${err.stack}`);
      }
      
      // Keep console logs for immediate debugging
      console.error('Failed to install update - Full error:', err);
      
      updateState = 'error';
      errorMessage = err?.message || 'Failed to install update';
      showNotification = true;
      startAutoHide();
    }
  }

  function dismissNotification() {
    showNotification = false;
    clearAutoHide();
  }

  function startAutoHide() {
    clearAutoHide();
    autoHideTimer = setTimeout(() => {
      showNotification = false;
    }, 5000);
  }

  function clearAutoHide() {
    if (autoHideTimer) {
      clearTimeout(autoHideTimer);
      autoHideTimer = null;
    }
  }

  $effect(() => {
    return () => clearAutoHide();
  });
</script>

{#if showNotification}
  <div class="fixed top-4 right-4 z-50 max-w-sm card shadow-lg overflow-hidden">
    <div class="p-4">
      <div class="flex items-start justify-between">
        <div class="flex-1 mr-3">
          {#if updateState === 'available'}
            <h3 class="text-sm font-medium">🦉 QueryOwl Update Available</h3>
            <p class="mt-1 text-sm text-surface-500">
              Version {update.version} is ready to install.
            </p>
          {:else if updateState === 'installing'}
            <h3 class="text-sm font-medium">📦 Installing Update</h3>
            <p class="mt-1 text-sm text-surface-500">
              {#if downloadProgress > 0}
                Downloading... {downloadProgress.toFixed(0)}%
              {:else}
                Preparing download...
              {/if}
            </p>
            <div class="mt-2 w-full bg-surface-300-600 rounded-full h-2 overflow-hidden">
              <div class="bg-primary-500 h-2 rounded-full transition-all duration-300" style="width: {downloadProgress}%"></div>
            </div>
          {:else if updateState === 'complete'}
            <h3 class="text-sm font-medium">✅ Update Complete</h3>
            <p class="mt-1 text-sm text-surface-500">
              Restarting QueryOwl...
            </p>
          {:else if updateState === 'error'}
            <h3 class="text-sm font-medium text-error-500">❌ Update Error</h3>
            <p class="mt-1 text-sm text-surface-500">
              {errorMessage}
            </p>
          {/if}
        </div>
        
        <button
          onclick={dismissNotification}
          class="flex-shrink-0 btn-ghost-surface rounded p-1"
          aria-label="Dismiss notification"
        >
          <svg class="w-4 h-4 text-surface-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
      
      {#if updateState === 'available'}
        <div class="mt-3 flex space-x-2">
          <button
            onclick={installUpdate}
            class="btn btn-filled-primary flex-1 text-sm py-2 px-3"
          >
            Install Now
          </button>
          <button
            onclick={dismissNotification}
            class="btn btn-ghost-surface flex-1 text-sm py-2 px-3"
          >
            Later
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}