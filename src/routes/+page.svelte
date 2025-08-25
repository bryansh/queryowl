<script>
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event) {
    event.preventDefault();
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="min-h-screen bg-gray-50 dark:bg-gray-900 flex items-center justify-center p-4">
  <div class="max-w-2xl w-full space-y-8">
    <h1 class="text-4xl font-bold text-center text-gray-900 dark:text-white">
      Welcome to QueryOwl
    </h1>

    <div class="flex justify-center space-x-4">
      <a href="https://tauri.app" target="_blank" class="p-4 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
        <img src="/tauri.svg" class="h-16 w-16" alt="Tauri Logo" />
      </a>
      <a href="https://svelte.dev" target="_blank" class="p-4 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">
        <img src="/svelte.svg" class="h-16 w-16" alt="Svelte Logo" />
      </a>
    </div>
    
    <p class="text-center text-gray-600 dark:text-gray-400">
      A database query tool built with Tauri and Svelte 5
    </p>

    <form onsubmit={greet} class="flex justify-center space-x-2">
      <input 
        id="greet-input" 
        placeholder="Enter a name..." 
        bind:value={name}
        class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
      />
      <button 
        type="submit"
        class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-gray-900"
      >
        Greet
      </button>
    </form>
    
    {#if greetMsg}
      <p class="text-center text-lg text-gray-700 dark:text-gray-300 mt-4">
        {greetMsg}
      </p>
    {/if}
  </div>
</main>