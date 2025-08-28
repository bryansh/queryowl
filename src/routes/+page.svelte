<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { Database, Settings, FileText } from "lucide-svelte";
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
  
  <!-- Sidebar -->
  <aside class="w-64 flex flex-col m-3 mr-1">
    <!-- Sidebar Header -->
    <div class="card px-4 py-6 mb-4">
      <h1 class="text-lg font-semibold flex items-center gap-2">
        🦉 QueryOwl
      </h1>
    </div>
    
    <!-- Navigation -->
    <nav class="card flex-1 p-4">
      <div class="space-y-2">
        <button
          class="btn w-full text-left px-4 py-3 text-sm {currentView === 'connections' ? 'btn-filled-primary' : 'btn-ghost-surface'}"
          onclick={() => currentView = "connections"}
        >
          <Database class="h-4 w-4 mr-3 inline" />
          Connections
        </button>
        <button
          class="btn w-full text-left px-4 py-3 text-sm {currentView === 'query' ? 'btn-filled-primary' : 'btn-ghost-surface'} {!$activeConnection ? 'opacity-50 cursor-not-allowed' : ''}"
          onclick={() => currentView = "query"}
          disabled={!$activeConnection}
          title={!$activeConnection ? "Connect to a database first" : "Run SQL queries"}
        >
          <FileText class="h-4 w-4 mr-3 inline" />
          Query Editor
        </button>
      </div>
      
      <!-- Connection Status in Sidebar -->
      <div class="mt-8">
        <ConnectionStatus />
      </div>
    </nav>
  </aside>

  <!-- Main content area -->
  <div class="flex-1 flex flex-col min-h-0 m-3 ml-1 overflow-hidden">
    <!-- Top toolbar -->
    <header class="card flex-shrink-0 flex items-center justify-between px-6 py-4 mb-4">
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
    <main class="card flex-1 p-6 overflow-hidden min-h-0">
    {#if currentView === "home"}
      <div class="flex-1 flex items-center justify-center">
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

