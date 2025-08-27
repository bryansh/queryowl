<script lang="ts">
	import { Database, Wifi, WifiOff, ChevronDown } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';
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
	<div class="flex items-center gap-3">
		{#if $connectionStatus.isConnected}
			<div class="flex items-center gap-2 text-green-600">
				<Wifi class="h-4 w-4" />
				<span class="text-sm font-medium">Connected to {$connectionStatus.activeConnection?.name}</span>
			</div>
			<Button size="sm" variant="outline" onclick={handleDisconnect}>
				Disconnect
			</Button>
		{:else}
			<div class="flex items-center gap-2 text-muted-foreground">
				<WifiOff class="h-4 w-4" />
				<span class="text-sm">Not connected</span>
			</div>
			{#if $connections.length > 0}
				<Button size="sm" variant="default" onclick={toggleQuickConnect}>
					<Database class="h-4 w-4 mr-2" />
					Quick Connect
					<ChevronDown class="h-4 w-4 ml-1" />
				</Button>
			{/if}
		{/if}

		{#if $connectionStatus.error}
			<div class="text-sm text-red-600">
				Error: {$connectionStatus.error}
			</div>
		{/if}
	</div>

	{#if showQuickConnect}
		<div class="absolute top-full left-0 mt-2 bg-card border rounded-lg shadow-lg min-w-80 z-50">
			<div class="p-3 border-b">
				<h3 class="font-medium text-foreground">Quick Connect</h3>
			</div>
			<div class="max-h-64 overflow-y-auto">
				{#each $connections as connection (connection.id)}
					<button 
						onclick={() => handleQuickConnect(connection)}
						class="w-full px-3 py-2 text-left hover:bg-accent transition-colors border-b last:border-b-0"
					>
						<div class="flex items-center gap-3">
							<Database class="h-4 w-4 text-muted-foreground" />
							<div class="flex-1">
								<div class="font-medium text-foreground">{connection.name}</div>
								<div class="text-sm text-muted-foreground">
									{connection.host}:{connection.port}/{connection.database}
								</div>
							</div>
						</div>
					</button>
				{/each}
			</div>
			<div class="p-3 border-t">
				<Button 
					size="sm" 
					variant="outline" 
					onclick={() => showQuickConnect = false}
					class="w-full"
				>
					Cancel
				</Button>
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