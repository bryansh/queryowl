<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { Database, Settings, FileText } from "lucide-svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import Input from "$lib/components/ui/input/input.svelte";
  import ConnectionManager from "$lib/components/ConnectionManager.svelte";
  import ConnectionStatus from "$lib/components/ConnectionStatus.svelte";
  import UpdateNotification from "$lib/components/UpdateNotification.svelte";
  import DebugShortcuts from "$lib/components/DebugShortcuts.svelte";
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

<div class="min-h-screen bg-background">
  <UpdateNotification />
  <DebugShortcuts />
  
  <!-- Header with navigation and connection status -->
  <header class="border-b bg-card">
    <div class="max-w-7xl mx-auto px-4 py-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-6">
          <h1 class="text-xl font-semibold text-foreground flex items-center gap-2">
            🦉 QueryOwl
          </h1>
          <nav class="flex items-center gap-4">
            <Button 
              variant={currentView === "home" ? "default" : "ghost"}
              size="sm"
              onclick={() => currentView = "home"}
            >
              Home
            </Button>
            <Button 
              variant={currentView === "connections" ? "default" : "ghost"}
              size="sm"
              onclick={() => currentView = "connections"}
            >
              <Database class="h-4 w-4 mr-2" />
              Connections
            </Button>
            <Button 
              variant={currentView === "query" ? "default" : "ghost"}
              size="sm"
              onclick={() => currentView = "query"}
              disabled={!$activeConnection}
              title={!$activeConnection ? "Connect to a database first" : "Run SQL queries"}
            >
              <FileText class="h-4 w-4 mr-2" />
              Query
            </Button>
          </nav>
        </div>
        <div class="flex items-center gap-4">
          <ConnectionStatus />
        </div>
      </div>
    </div>
  </header>

  <!-- Main content -->
  <main class="max-w-7xl mx-auto px-4 py-8">
    {#if currentView === "home"}
      <div class="max-w-2xl mx-auto space-y-8 text-center">
        <div class="space-y-4">
          <h2 class="text-3xl font-bold text-foreground">
            Welcome to QueryOwl 🦉
          </h2>
          <p class="text-lg text-muted-foreground">
            A powerful PostgreSQL database query tool built with Tauri and Svelte 5
          </p>
        </div>

        <div class="flex justify-center space-x-4">
          <a href="https://tauri.app" target="_blank" class="p-4 rounded-lg hover:bg-accent transition-colors">
            <img src="/tauri.svg" class="h-16 w-16" alt="Tauri Logo" />
          </a>
          <a href="https://svelte.dev" target="_blank" class="p-4 rounded-lg hover:bg-accent transition-colors">
            <img src="/svelte.svg" class="h-16 w-16" alt="Svelte Logo" />
          </a>
        </div>
        
        <div class="space-y-4">
          <h3 class="text-xl font-semibold text-foreground">Get Started</h3>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 max-w-lg mx-auto">
            <Button 
              onclick={() => currentView = "connections"}
              class="h-20 flex-col"
            >
              <Database class="h-6 w-6 mb-2" />
              Manage Connections
            </Button>
            <Button 
              variant="outline"
              class="h-20 flex-col"
              onclick={() => currentView = "query"}
              disabled={!$activeConnection}
            >
              <FileText class="h-6 w-6 mb-2" />
              Run Queries
              {#if !$activeConnection}
                <span class="text-xs text-muted-foreground mt-1">Connect to database first</span>
              {/if}
            </Button>
          </div>
        </div>


        <!-- Demo greeting form -->
        <div class="border-t pt-8">
          <h3 class="text-lg font-medium text-foreground mb-4">Test Greeting</h3>
          <form onsubmit={greet} class="flex justify-center space-x-2">
            <Input 
              id="greet-input" 
              placeholder="Enter a name..." 
              bind:value={name}
            />
            <Button type="submit">
              Greet
            </Button>
          </form>
          
          {#if greetMsg}
            <p class="text-center text-lg text-foreground mt-4">
              {greetMsg}
            </p>
          {/if}
        </div>
      </div>
    {:else if currentView === "connections"}
      <ConnectionManager />
    {:else if currentView === "query"}
      <QueryInterface activeConnection={$activeConnection} />
    {/if}
  </main>
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

