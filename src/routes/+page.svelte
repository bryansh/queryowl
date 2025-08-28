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

<div class="w-full h-full bg-background flex overflow-hidden">
  <UpdateNotification />
  
  <!-- Sidebar like Beekeeper Studio -->
  <aside class="beekeeper-sidebar w-64 flex flex-col m-3 mr-1 rounded-lg">
    <!-- Sidebar Header -->
    <div class="px-4 py-6 border-b border-slate-600">
      <h1 class="text-lg font-semibold text-white flex items-center gap-2">
        🦉 QueryOwl
      </h1>
    </div>
    
    <!-- Navigation -->
    <nav class="flex-1 p-4">
      <div class="space-y-2">
        <button
          class="w-full text-left px-4 py-3 rounded-lg text-sm transition-colors {currentView === 'connections' ? 'bg-blue-600 text-white shadow-lg' : 'hover:bg-slate-600 text-slate-300'}"
          onclick={() => currentView = "connections"}
        >
          <Database class="h-4 w-4 mr-3 inline" />
          Connections
        </button>
        <button
          class="w-full text-left px-4 py-3 rounded-lg text-sm transition-colors {currentView === 'query' ? 'bg-blue-600 text-white shadow-lg' : 'hover:bg-slate-600 text-slate-300'} {!$activeConnection ? 'opacity-50 cursor-not-allowed' : ''}"
          onclick={() => currentView = "query"}
          disabled={!$activeConnection}
          title={!$activeConnection ? "Connect to a database first" : "Run SQL queries"}
        >
          <FileText class="h-4 w-4 mr-3 inline" />
          Query Editor
        </button>
      </div>
      
      <!-- Connection Status in Sidebar -->
      <div class="mt-8 p-4 bg-slate-700 rounded-lg border border-slate-600">
        <ConnectionStatus />
      </div>
    </nav>
  </aside>

  <!-- Main content area -->
  <div class="flex-1 flex flex-col min-h-0 m-3 ml-1 rounded-lg overflow-hidden">
    <!-- Top toolbar -->
    <header class="flex-shrink-0 bg-slate-800 flex items-center justify-between px-6 py-4 border-b border-slate-600">
      <div class="flex items-center gap-4">
        {#if currentView === "query" && $activeConnection}
          <span class="text-sm text-slate-300 font-medium">
            {$activeConnection.name} • {$activeConnection.database}
          </span>
        {:else if currentView === "connections"}
          <span class="text-sm text-slate-300 font-medium">Manage Connections</span>
        {:else}
          <span class="text-sm text-slate-300 font-medium">Welcome to QueryOwl</span>
        {/if}
      </div>
      <div class="flex items-center gap-2">
        <!-- Add any toolbar actions here if needed -->
      </div>
    </header>

    <!-- Main content -->
    <main class="flex-1 p-6 overflow-hidden min-h-0 bg-slate-900">
    {#if currentView === "home"}
      <div class="flex-1 flex items-center justify-center">
        <div class="max-w-md text-center space-y-8">
          <div class="space-y-4">
            <div class="text-6xl mb-4">🦉</div>
            <h2 class="text-2xl font-bold text-slate-200">
              Welcome to QueryOwl
            </h2>
            <p class="text-slate-400">
              A powerful PostgreSQL database query tool
            </p>
          </div>
          
          <div class="space-y-3">
            <button 
              onclick={() => currentView = "connections"}
              class="btn btn-primary w-full py-3 flex items-center justify-center gap-3"
            >
              <Database class="h-5 w-5" />
              Manage Connections
            </button>
            <button 
              onclick={() => currentView = "query"}
              disabled={!$activeConnection}
              class="btn btn-outline w-full py-3 flex items-center justify-center gap-3 {!$activeConnection ? 'btn-disabled' : ''}"
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
  <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 9999;">
    <div style="background: hsl(var(--card)); border: 1px solid hsl(var(--border)); border-radius: 8px; padding: 24px; max-width: 500px; margin: 16px; box-shadow: 0 20px 50px rgba(0,0,0,0.4);">
      <h3 style="font-size: 18px; font-weight: 600; margin-bottom: 16px; color: hsl(var(--foreground));">Log File Path</h3>
      <div style="background: hsl(var(--muted)); padding: 12px; border-radius: 6px; font-family: ui-monospace, 'SF Mono', Consolas, monospace; font-size: 13px; word-break: break-all; margin-bottom: 16px; color: hsl(var(--foreground)); border: 1px solid hsl(var(--border));">
        {logPath}
      </div>
      <div style="display: flex; gap: 12px;">
        <button onclick={copyLogPath} style="background: hsl(var(--primary)); color: hsl(var(--primary-foreground)); padding: 8px 16px; border: none; border-radius: 6px; cursor: pointer; font-weight: 500; transition: all 0.2s;">
          Copy Path
        </button>
        <button onclick={() => showLogPath = false} style="background: transparent; color: hsl(var(--foreground)); padding: 8px 16px; border: 1px solid hsl(var(--border)); border-radius: 6px; cursor: pointer; font-weight: 500; transition: all 0.2s;">
          Close
        </button>
      </div>
    </div>
  </div>
{/if}

