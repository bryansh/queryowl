<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import { Database, Settings, FileText, Menu, History, Table, Clock, Bookmark } from "lucide-svelte";
  import { Navigation } from '@skeletonlabs/skeleton-svelte';
  import ConnectionManager from "$lib/components/ConnectionManager.svelte";
  import ConnectionStatus from "$lib/components/ConnectionStatus.svelte";
  import UpdateNotification from "$lib/components/UpdateNotification.svelte";
  import QueryInterface from "$lib/components/QueryInterface.svelte";
  import QueryHistory from "$lib/components/QueryHistory.svelte";
  import SavedQueries from "$lib/components/SavedQueries.svelte";
  import { connections, activeConnection, disconnectFromDatabase as disconnectDB } from '$lib/stores/connections';

  let name = $state("");
  let greetMsg = $state("");
  let currentView = $state("connections"); // "connections", "query", "history", "saved"
  let showLogPath = $state(false);
  let logPath = $state("");
  let isNavExpanded = $state(false);
  let showStatusMenu = $state(false);
  let queryInterface;
  
  // Query results state for status bar
  let queryResults = $state(null);
  let queryError = $state(null);
  let executionTime = $state(null);
  let isExecuting = $state(false);

  onMount(async () => {
    // Restore window state on app launch
    try {
      await invoke('restore_window_state');
    } catch (error) {
      console.warn('Failed to restore window state:', error);
    }
    
    // Listen for menu events
    const unlistenLogPath = await listen("show_log_path", (event) => {
      logPath = event.payload;
      showLogPath = true;
    });

    // Close status menu when clicking outside
    const handleClickOutside = (event) => {
      if (showStatusMenu && !event.target.closest('.status-menu-container')) {
        showStatusMenu = false;
      }
    };
    
    document.addEventListener('click', handleClickOutside);
    
    // Save window state on window events
    let saveTimeout = null;
    const saveWindowState = () => {
      if (saveTimeout) clearTimeout(saveTimeout);
      saveTimeout = window.setTimeout(async () => {
        try {
          await invoke('save_window_state');
        } catch (error) {
          console.warn('Failed to save window state:', error);
        }
      }, 500); // Debounce saves
    };
    
    // Listen for window resize and move events
    window.addEventListener('resize', saveWindowState);
    
    // Listen to Tauri window events for more accurate state tracking
    const currentWindow = getCurrentWebviewWindow();
    const unlistenMoved = await currentWindow.listen('tauri://move', saveWindowState);
    const unlistenResized = await currentWindow.listen('tauri://resize', saveWindowState);
    
    // Save state when app is about to close
    const handleBeforeUnload = async () => {
      try {
        await invoke('save_window_state');
      } catch (error) {
        console.warn('Failed to save window state on close:', error);
      }
    };
    
    window.addEventListener('beforeunload', handleBeforeUnload);

    return () => {
      unlistenLogPath();
      unlistenMoved();
      unlistenResized();
      document.removeEventListener('click', handleClickOutside);
      window.removeEventListener('resize', saveWindowState);
      window.removeEventListener('beforeunload', handleBeforeUnload);
      if (saveTimeout) clearTimeout(saveTimeout);
    };
  });

  async function greet(event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }

  function copyLogPath() {
    navigator.clipboard.writeText(logPath);
    showLogPath = false;
  }

  async function handleDisconnect() {
    try {
      await disconnectDB();
      showStatusMenu = false;
    } catch (error) {
      console.error('Failed to disconnect:', error);
    }
  }

  // Handle running a query from history
  async function handleRunQueryFromHistory(sql) {
    // Switch to query view
    currentView = "query";
    // Wait for the next tick to ensure the component is mounted and reactive
    await new Promise(resolve => {
      const checkReady = () => {
        if (queryInterface && queryInterface.loadAndRunQuery) {
          resolve();
        } else {
          requestAnimationFrame(checkReady);
        }
      };
      checkReady();
    });
    // Load and execute the query
    await queryInterface.loadAndRunQuery(sql);
  }

  // Handle editing a query from history (load without running)
  async function handleEditQueryFromHistory(sql) {
    // Switch to query view
    currentView = "query";
    // Wait for the next tick to ensure the component is mounted and reactive
    await new Promise(resolve => {
      const checkReady = () => {
        if (queryInterface && queryInterface.loadQuery) {
          resolve();
        } else {
          requestAnimationFrame(checkReady);
        }
      };
      checkReady();
    });
    // Load the query without executing
    queryInterface.loadQuery(sql);
  }
</script>

<div class="w-full h-full flex overflow-hidden">
  <UpdateNotification />
  
  <!-- Navigation Rail -->
  <Navigation.Rail 
    expanded={false} 
    value={currentView}
    onValueChange={(newValue) => currentView = newValue || "connections"}
    classes="m-3 mr-1"
  >
    {#snippet header()}
      <Navigation.Tile 
        id="connections"
        labelExpanded="Connections"
        onclick={() => currentView = "connections"}
        classes="flex items-center justify-center"
      >
        <Database />
      </Navigation.Tile>
      <Navigation.Tile 
        id="query"
        labelExpanded="Query Editor"
        onclick={() => currentView = "query"}
        disabled={!$activeConnection}
        classes={`flex items-center justify-center ${!$activeConnection ? 'opacity-50 cursor-not-allowed' : ''}`}
      >
        <FileText />
      </Navigation.Tile>
      <Navigation.Tile 
        id="history"
        labelExpanded="Query History"
        onclick={() => currentView = "history"}
        disabled={!$activeConnection}
        classes={`flex items-center justify-center ${!$activeConnection ? 'opacity-50 cursor-not-allowed' : ''}`}
      >
        <History />
      </Navigation.Tile>
      <Navigation.Tile 
        id="saved"
        labelExpanded="Saved Queries"
        onclick={() => currentView = "saved"}
        disabled={!$activeConnection}
        classes={`flex items-center justify-center ${!$activeConnection ? 'opacity-50 cursor-not-allowed' : ''}`}
      >
        <Bookmark />
      </Navigation.Tile>
    {/snippet}
    
    {#snippet tiles()}
    {/snippet}
  </Navigation.Rail>

  <!-- Main content area -->
  <div class="flex-1 flex flex-col min-h-0 m-3 ml-1 overflow-hidden">
    <!-- Top toolbar -->
    <header class="card flex-shrink-0 flex items-center justify-between px-6 py-2 mb-2">
      <div class="flex items-center gap-4">
        {#if currentView === "connections"}
          <div class="flex items-center gap-3">
            <Database class="h-8 w-8 text-primary-500" />
            <h1 class="text-3xl font-semibold">Connection Manager</h1>
          </div>
        {:else if currentView === "query"}
          <div class="flex items-center gap-3">
            <FileText class="h-8 w-8 text-primary-500" />
            <h1 class="text-3xl font-semibold">Query Editor</h1>
          </div>
        {:else if currentView === "history"}
          <div class="flex items-center gap-3">
            <History class="h-8 w-8 text-primary-500" />
            <h1 class="text-3xl font-semibold">Query History</h1>
          </div>
        {:else if currentView === "saved"}
          <div class="flex items-center gap-3">
            <Bookmark class="h-8 w-8 text-primary-500" />
            <h1 class="text-3xl font-semibold">Saved Queries</h1>
          </div>
        {/if}
      </div>
      <div class="flex items-center gap-2">
        {#if currentView === "query" && $activeConnection}
          <div class="flex items-center gap-2 text-surface-500">
            <Database class="h-5 w-5" style="color: {$activeConnection.color || '#14b8a6'}" />
            <span class="text-lg font-medium">{$activeConnection.name}</span>
            <span>•</span>
            <span class="text-base">{$activeConnection.database}</span>
          </div>
        {/if}
      </div>
    </header>

    <!-- Main content -->
    <main class="card flex-1 overflow-auto min-h-0">
    {#if currentView === "connections"}
      <ConnectionManager />
    {:else if currentView === "query"}
      <QueryInterface 
        bind:this={queryInterface} 
        activeConnection={$activeConnection} 
        onResultsChange={(results, error, time, executing) => {
          queryResults = results;
          queryError = error;
          executionTime = time;
          isExecuting = executing;
        }}
      />
    {:else if currentView === "history"}
      <QueryHistory activeConnection={$activeConnection} onRunQuery={handleRunQueryFromHistory} onEditQuery={handleEditQueryFromHistory} />
    {:else if currentView === "saved"}
      <SavedQueries activeConnection={$activeConnection} onRunQuery={handleRunQueryFromHistory} onEditQuery={handleEditQueryFromHistory} />
    {/if}
    </main>
    
    <!-- Status bar - only show when connected -->
    {#if $activeConnection}
      <div class="relative status-menu-container">
        <div 
          style="background-color: {$activeConnection.color || '#14b8a6'};" 
          class="text-white px-4 py-2 flex items-center justify-between text-sm font-medium mt-2 animate-slide-up cursor-pointer hover:opacity-90 transition-opacity"
          onclick={() => showStatusMenu = !showStatusMenu}
        >
          <div class="flex items-center gap-3">
            <Database class="h-4 w-4" />
            <span>{$activeConnection.name}</span>
            {#if currentView === "query" && queryResults && queryResults.length > 0}
              <span>•</span>
              {#if queryResults.length === 1 && queryResults[0].status === 'success'}
                <!-- DDL/DML success message -->
                <span class="text-primary-300">{queryResults[0].query_type} successful</span>
                {#if queryResults[0].affected_rows > 0}
                  <span>•</span>
                  <span>{queryResults[0].affected_rows} {queryResults[0].affected_rows === 1 ? 'row' : 'rows'} affected</span>
                {/if}
              {:else}
                <!-- Regular SELECT results -->
                <div class="flex items-center gap-1">
                  <Table class="h-3 w-3" />
                  <span>{queryResults.length.toLocaleString()} {queryResults.length === 1 ? 'row' : 'rows'}</span>
                </div>
              {/if}
              {#if executionTime !== null}
                <span>•</span>
                <div class="flex items-center gap-1">
                  <Clock class="h-3 w-3" />
                  <span>{executionTime}ms</span>
                </div>
              {/if}
            {:else if currentView === "query" && queryError}
              <span>•</span>
              <span class="text-red-300">Query failed</span>
              {#if executionTime !== null}
                <span>•</span>
                <div class="flex items-center gap-1">
                  <Clock class="h-3 w-3" />
                  <span>{executionTime}ms</span>
                </div>
              {/if}
            {:else if currentView === "query" && isExecuting}
              <span>•</span>
              <span>Executing...</span>
            {/if}
          </div>
          <div class="flex items-center gap-2 text-xs">
            {#if currentView === "query" && queryResults && queryResults.length > 0 && !(queryResults.length === 1 && queryResults[0].status === 'success')}
              <!-- Copy and Export buttons - only show for actual tabular results -->
              <button 
                onclick={() => queryInterface?.copyResults()}
                class="hover:bg-white/20 rounded px-2 py-1 transition-colors"
                title="Copy results"
              >
                Copy
              </button>
              <button 
                onclick={() => queryInterface?.exportResults()}
                class="hover:bg-white/20 rounded px-2 py-1 transition-colors"
                title="Export as CSV"
              >
                CSV
              </button>
              <span class="mx-1">|</span>
            {/if}
            <span>{currentView === "query" && isExecuting ? "Executing" : "Ready"}</span>
            <span class="ml-2">⌄</span>
          </div>
        </div>
        
        <!-- Context Menu -->
        {#if showStatusMenu}
          <div class="absolute bottom-full right-0 mb-2 bg-surface-100-900 border border-surface-300-600 rounded-lg shadow-lg z-50 min-w-[200px]">
            <div class="py-2">
              <button 
                onclick={handleDisconnect}
                class="w-full px-4 py-2 text-left text-sm hover:bg-surface-200-700 transition-colors flex items-center gap-3"
              >
                <span class="text-red-500">⏻</span>
                Disconnect from {$activeConnection.name}
              </button>
              <hr class="border-surface-300-600 my-1" />
              <div class="px-4 py-2 text-xs text-surface-500">
                <div>Database: {$activeConnection.database}</div>
                <div>Host: {$activeConnection.host}:{$activeConnection.port}</div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    {/if}
    
  </div>
</div>


<!-- Log Path Dialog from Native Menu -->
{#if showLogPath}
  <div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
    <div class="card p-6 max-w-lg mx-4 shadow-2xl">
      <h3 class="text-lg font-semibold mb-4">Log File Path</h3>
      <div class="bg-surface-100-900 p-3 rounded-md font-mono text-sm break-all mb-4 border">
        {logPath}
      </div>
      <div class="flex gap-3">
        <button onclick={copyLogPath} class="btn btn-filled-primary">
          Copy Path
        </button>
        <button onclick={() => showLogPath = false} class="btn btn-ghost-surface">
          Close
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(.animate-fade-in) {
    animation: fadeIn 0.3s ease-in-out;
  }
  
  @keyframes fadeIn {
    from { 
      opacity: 0; 
      transform: translateX(-10px); 
    }
    to { 
      opacity: 1; 
      transform: translateX(0); 
    }
  }
  
  /* Force vertical stability for Navigation Tiles */
  :global([data-navigation-tile]) {
    height: 48px;
    min-height: 48px;
    max-height: 48px;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    padding: 12px;
  }
  
  :global([data-navigation-tile] svg) {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }
  
  /* Status bar slide-up animation */
  .animate-slide-up {
    animation: slideUp 0.3s ease-out;
  }
  
  @keyframes slideUp {
    from { 
      transform: translateY(100%);
      opacity: 0;
    }
    to { 
      transform: translateY(0);
      opacity: 1;
    }
  }
  
</style>

