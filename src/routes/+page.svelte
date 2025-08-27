<script>
  import { invoke } from "@tauri-apps/api/core";
  import Button from "$lib/components/ui/button/button.svelte";
  import Input from "$lib/components/ui/input/input.svelte";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="min-h-screen bg-background flex items-center justify-center p-4">
  <div class="max-w-2xl w-full space-y-8">
    <h1 class="text-4xl font-bold text-center text-foreground">
      Welcome to QueryOwl 🦉
    </h1>

    <div class="flex justify-center space-x-4">
      <a href="https://tauri.app" target="_blank" class="p-4 rounded-lg hover:bg-accent transition-colors">
        <img src="/tauri.svg" class="h-16 w-16" alt="Tauri Logo" />
      </a>
      <a href="https://svelte.dev" target="_blank" class="p-4 rounded-lg hover:bg-accent transition-colors">
        <img src="/svelte.svg" class="h-16 w-16" alt="Svelte Logo" />
      </a>
    </div>
    
    <p class="text-center text-muted-foreground">
      A database query tool built with Tauri and Svelte 5
    </p>

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
</main>