<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { Database, Settings, FileText, Menu } from "lucide-svelte";
  import { Navigation } from '@skeletonlabs/skeleton-svelte';
  import ConnectionManager from "$lib/components/ConnectionManager.svelte";
  import ConnectionStatus from "$lib/components/ConnectionStatus.svelte";
  import UpdateNotification from "$lib/components/UpdateNotification.svelte";
  import QueryInterface from "$lib/components/QueryInterface.svelte";
  import { connections, activeConnection } from '$lib/stores/connections';

  let name = $state("");
  let greetMsg = $state("");
  let currentView = $state("home"); // "home", "connections", "query"
  let showLogPath = $state(false);
  let logPath = $state("");
  let isNavExpanded = $state(false);

  onMount(async () => {
    // Listen for menu events
    const unlistenLogPath = await listen("show_log_path", (event) => {
      logPath = event.payload;
      showLogPath = true;
    });

    return () => {
      unlistenLogPath();
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
</script>

<div class="w-full h-full flex overflow-hidden">
  <UpdateNotification />
  
  <!-- Navigation Rail -->
  <Navigation.Rail 
    expanded={false} 
    value={currentView === "home" ? "" : currentView}
    onValueChange={(newValue) => currentView = newValue || "home"}
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
    {/snippet}
    
    {#snippet tiles()}
    {/snippet}
  </Navigation.Rail>

  <!-- Main content area -->
  <div class="flex-1 flex flex-col min-h-0 m-3 ml-1 overflow-hidden">
    <!-- Top toolbar -->
    <header class="card flex-shrink-0 flex items-center justify-between px-6 py-2 mb-2">
      <div class="flex items-center gap-4">
        {#if currentView === "query" && $activeConnection}
          <span class="text-sm font-medium">
            {$activeConnection.name} • {$activeConnection.database}
          </span>
        {:else if currentView === "connections"}
          <span class="text-sm font-medium">Manage Connections</span>
        {:else}
          <span class="text-sm font-medium">Welcome to QueryOwl</span>
        {/if}
      </div>
      <div class="flex items-center gap-2">
        <!-- Add any toolbar actions here if needed -->
      </div>
    </header>

    <!-- Main content -->
    <main class="card flex-1 overflow-hidden min-h-0">
    {#if currentView === "home"}
      <div class="flex-1 flex items-center justify-center p-6">
        <div class="max-w-md text-center space-y-8">
          <div class="space-y-4">
            <div class="text-6xl mb-4">🦉</div>
            <h2 class="text-2xl font-bold">
              Welcome to QueryOwl
            </h2>
            <p class="text-surface-500">
              A powerful PostgreSQL database query tool
            </p>
          </div>
          
          <div class="space-y-3">
            <button 
              onclick={() => currentView = "connections"}
              class="btn btn-filled-primary w-full py-3 flex items-center justify-center gap-3"
            >
              <Database class="h-5 w-5" />
              Manage Connections
            </button>
            <button 
              onclick={() => currentView = "query"}
              disabled={!$activeConnection}
              class="btn btn-ghost-surface w-full py-3 flex items-center justify-center gap-3 {!$activeConnection ? 'opacity-50 cursor-not-allowed' : ''}"
            >
              <FileText class="h-5 w-5" />
              {!$activeConnection ? "Connect to database first" : "Open Query Editor"}
            </button>
          </div>
        </div>
      </div>
    {:else if currentView === "connections"}
      <ConnectionManager />
    {:else if currentView === "query"}
      <QueryInterface activeConnection={$activeConnection} />
    {/if}
    </main>
    
  </div>
</div>

<!-- Floating Status bar at bottom like Beekeeper Studio -->
<div 
  style="position: fixed; bottom: 16px; left: 16px; right: 16px; z-index: 1000; background-color: {$activeConnection?.color || '#22c55e'};" 
  class="text-white px-4 py-2 flex items-center justify-between text-sm font-medium rounded-lg shadow-lg"
>
  <div class="flex items-center gap-3">
    {#if $activeConnection}
      <span>🦉 QueryOwl</span>
      <span>•</span>
      <span>Connected to {$activeConnection.name}</span>
    {:else}
      <span>🦉 QueryOwl</span>
      <span>•</span>
      <span>No database connected</span>
    {/if}
  </div>
  <div class="flex items-center gap-2 text-xs">
    <span>Ready</span>
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
  
</style>

