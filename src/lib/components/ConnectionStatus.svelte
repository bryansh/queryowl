<script lang="ts">
	import { Database, Wifi, WifiOff, ChevronDown } from 'lucide-svelte';
	import { connectionStatus, connections, loadConnections, connectToDatabase, disconnectFromDatabase } from '$lib/stores/connections';
	import type { DatabaseConnection } from '$lib/types/database';
	import { onMount } from 'svelte';

	let showQuickConnect = $state(false);

	onMount(() => {
		loadConnections();
	});

	async function handleQuickConnect(connection: DatabaseConnection) {
		try {
			await connectToDatabase(connection);
			showQuickConnect = false;
		} catch (error) {
			console.error('Failed to connect:', error);
		}
	}

	async function handleDisconnect() {
		try {
			await disconnectFromDatabase();
		} catch (error) {
			console.error('Failed to disconnect:', error);
		}
	}

	function toggleQuickConnect() {
		showQuickConnect = !showQuickConnect;
	}
</script>

<div class="relative">
	<div class="flex flex-col gap-3">
		{#if $connectionStatus.isConnected}
			<div class="flex items-center gap-2 text-success-500">
				<Wifi class="h-4 w-4" />
				<span class="text-sm font-medium">Connected to {$connectionStatus.activeConnection?.name}</span>
			</div>
			<button onclick={handleDisconnect} class="btn btn-sm btn-ghost-surface">
				Disconnect
			</button>
		{:else}
			<div class="flex items-center gap-2 text-surface-500">
				<WifiOff class="h-4 w-4" />
				<span class="text-sm">Not connected</span>
			</div>
			{#if $connections.length > 0}
				<button onclick={toggleQuickConnect} class="btn btn-sm btn-filled-primary">
					<Database class="h-4 w-4 mr-2" />
					Quick Connect
					<ChevronDown class="h-4 w-4 ml-1" />
				</button>
			{/if}
		{/if}

		{#if $connectionStatus.error}
			<div class="text-sm text-error-500">
				Error: {$connectionStatus.error}
			</div>
		{/if}
	</div>

	{#if showQuickConnect}
		<div class="card absolute top-full left-0 mt-2 shadow-lg min-w-80 z-50">
			<div class="p-3 border-b border-surface-300-600">
				<h3 class="font-medium">Quick Connect</h3>
			</div>
			<div class="max-h-64 overflow-y-auto">
				{#each $connections as connection (connection.id)}
					<button 
						onclick={() => handleQuickConnect(connection)}
						class="w-full px-3 py-2 text-left hover:bg-surface-200-700 transition-colors border-b border-surface-300-600 last:border-b-0"
					>
						<div class="flex items-center gap-3">
							<Database class="h-4 w-4 text-surface-500" />
							<div class="flex-1">
								<div class="font-medium">{connection.name}</div>
								<div class="text-sm text-surface-500">
									{connection.host}:{connection.port}/{connection.database}
								</div>
							</div>
						</div>
					</button>
				{/each}
			</div>
			<div class="p-3 border-t border-surface-300-600">
				<button 
					onclick={() => showQuickConnect = false}
					class="btn btn-sm btn-ghost-surface w-full"
				>
					Cancel
				</button>
			</div>
		</div>
	{/if}
</div>

<!-- Click outside to close -->
{#if showQuickConnect}
	<!-- svelte-ignore a11y-click-events-have-key-events -->
	<!-- svelte-ignore a11y-no-static-element-interactions -->
	<div class="fixed inset-0 z-40" onclick={() => showQuickConnect = false}></div>
{/if}