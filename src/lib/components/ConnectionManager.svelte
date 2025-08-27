<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Database, Plus, Trash2, TestTube, Loader2, AlertCircle, CheckCircle, Edit } from 'lucide-svelte';
	import { connections, loadConnections, saveConnection, updateConnection, deleteConnection, testConnection, connectToDatabase } from '$lib/stores/connections';
	import type { CreateConnectionRequest, UpdateConnectionRequest, TestConnectionRequest, TestConnectionResponse } from '$lib/types/database';
	import { invoke } from '@tauri-apps/api/core';

	let showForm = $state(false);
	let editingId = $state<string | null>(null);
	let isLoading = $state(false);
	let testResults = $state<Record<string, { success: boolean; error?: string }>>({});
	let deleteConfirm = $state<{ show: boolean; id: string | null }>({ show: false, id: null });
	
	let formData = $state<CreateConnectionRequest>({
		name: '',
		host: 'localhost',
		port: 5432,
		database: '',
		username: '',
		password: '',
		ssl: false
	});

	onMount(() => {
		loadConnections();
	});

	async function handleSubmit(event) {
		event.preventDefault();
		if (!formData.name || !formData.host || !formData.database || !formData.username || !formData.password) {
			return;
		}
		
		isLoading = true;
		try {
			if (editingId) {
				const updateRequest: UpdateConnectionRequest = {
					id: editingId,
					...formData
				};
				await updateConnection(updateRequest);
			} else {
				await saveConnection(formData);
			}
			resetForm();
			showForm = false;
			editingId = null;
		} catch (error) {
			console.error('Failed to save connection:', error);
		} finally {
			isLoading = false;
		}
	}

	function handleDelete(id: string) {
		deleteConfirm = { show: true, id };
	}
	
	async function confirmDelete() {
		if (deleteConfirm.id) {
			try {
				await deleteConnection(deleteConfirm.id);
			} catch (error) {
				console.error('Failed to delete connection:', error);
			}
		}
		deleteConfirm = { show: false, id: null };
	}
	
	function cancelDelete() {
		deleteConfirm = { show: false, id: null };
	}

	async function handleTest(connection: any) {
		testResults[connection.id] = { success: false };
		
		try {
			// Use the new test_stored_connection command that handles encrypted passwords
			const result = await invoke<TestConnectionResponse>('test_stored_connection', { connection });
			testResults[connection.id] = result;
		} catch (error) {
			testResults[connection.id] = { success: false, error: String(error) };
		}
	}

	async function handleConnect(connection: any) {
		try {
			await connectToDatabase(connection);
		} catch (error) {
			console.error('Failed to connect:', error);
		}
	}

	function handleEdit(connection: any) {
		formData = {
			name: connection.name,
			host: connection.host,
			port: connection.port,
			database: connection.database,
			username: connection.username,
			password: '', // Clear password field for security - user must re-enter
			ssl: connection.ssl || false
		};
		editingId = connection.id;
		showForm = true;
	}

	function resetForm() {
		formData = {
			name: '',
			host: 'localhost',
			port: 5432,
			database: '',
			username: '',
			password: '',
			ssl: false
		};
		editingId = null;
	}

	function formatLastConnected(date?: string) {
		if (!date) return 'Never';
		return new Date(date).toLocaleDateString();
	}
</script>

<div class="p-6 max-w-4xl mx-auto">
	<div class="flex items-center justify-between mb-6">
		<div class="flex items-center gap-3">
			<Database class="h-6 w-6 text-primary" />
			<h1 class="text-2xl font-semibold text-foreground">Connection Manager</h1>
		</div>
		<Button onclick={() => showForm = !showForm} variant="default">
			<Plus class="h-4 w-4 mr-2" />
			Add Connection
		</Button>
	</div>

	{#if showForm}
		<div class="bg-card border rounded-lg p-6 mb-6">
			<h2 class="text-lg font-medium mb-4 text-foreground">
				{editingId ? 'Edit PostgreSQL Connection' : 'New PostgreSQL Connection'}
			</h2>
			<form onsubmit={handleSubmit} class="space-y-4">
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<div>
						<label class="block text-sm font-medium text-foreground mb-2">Connection Name</label>
						<Input 
							bind:value={formData.name} 
							placeholder="My Database"
							required
						/>
					</div>
					<div>
						<label class="block text-sm font-medium text-foreground mb-2">Database Name</label>
						<Input 
							bind:value={formData.database} 
							placeholder="myapp_production"
							required
						/>
					</div>
					<div>
						<label class="block text-sm font-medium text-foreground mb-2">Host</label>
						<Input 
							bind:value={formData.host} 
							placeholder="localhost"
							required
						/>
					</div>
					<div>
						<label class="block text-sm font-medium text-foreground mb-2">Port</label>
						<Input 
							bind:value={formData.port} 
							type="number"
							placeholder="5432"
							required
						/>
					</div>
					<div>
						<label class="block text-sm font-medium text-foreground mb-2">Username</label>
						<Input 
							bind:value={formData.username} 
							placeholder="postgres"
							required
						/>
					</div>
					<div>
						<label class="block text-sm font-medium text-foreground mb-2">Password</label>
						<Input 
							bind:value={formData.password} 
							type="password"
							placeholder={editingId ? "Enter password (required)" : "••••••••"}
							required
						/>
					</div>
				</div>
				<div class="flex items-center gap-3">
					<input 
						type="checkbox" 
						bind:checked={formData.ssl} 
						id="ssl"
						class="rounded border-input"
					/>
					<label for="ssl" class="text-sm text-foreground">Use SSL</label>
				</div>
				<div class="flex gap-3 pt-4">
					<Button type="submit" disabled={isLoading}>
						{#if isLoading}
							<Loader2 class="h-4 w-4 mr-2 animate-spin" />
						{/if}
						{editingId ? 'Update Connection' : 'Save Connection'}
					</Button>
					<Button type="button" variant="outline" onclick={() => { showForm = false; resetForm(); }}>
						Cancel
					</Button>
				</div>
			</form>
		</div>
	{/if}

	<div class="space-y-4">
		{#each $connections as connection (connection.id)}
			<div class="bg-card border rounded-lg p-4">
				<div class="flex items-center justify-between">
					<div class="flex-1">
						<div class="flex items-center gap-3 mb-2">
							<Database class="h-5 w-5 text-muted-foreground" />
							<h3 class="font-medium text-foreground">{connection.name}</h3>
						</div>
						<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-2 text-sm text-muted-foreground">
							<div>Host: {connection.host}:{connection.port}</div>
							<div>Database: {connection.database}</div>
							<div>User: {connection.username}</div>
							<div>Last connected: {formatLastConnected(connection.last_connected)}</div>
						</div>
						{#if testResults[connection.id]}
							<div class="mt-2 flex items-center gap-2">
								{#if testResults[connection.id].success}
									<CheckCircle class="h-4 w-4 text-green-500" />
									<span class="text-sm text-green-600">Connection successful</span>
								{:else}
									<AlertCircle class="h-4 w-4 text-red-500" />
									<span class="text-sm text-red-600">
										{testResults[connection.id].error || 'Connection failed'}
									</span>
								{/if}
							</div>
						{/if}
					</div>
					<div class="flex items-center gap-2">
						<Button 
							size="sm" 
							variant="outline"
							onclick={() => handleTest(connection)}
							title="Test Connection"
						>
							<TestTube class="h-4 w-4" />
						</Button>
						<Button 
							size="sm" 
							variant="default"
							onclick={() => handleConnect(connection)}
						>
							Connect
						</Button>
						<Button 
							size="sm" 
							variant="outline"
							onclick={() => handleEdit(connection)}
							title="Edit Connection"
						>
							<Edit class="h-4 w-4" />
						</Button>
						<Button 
							size="sm" 
							variant="outline"
							onclick={() => handleDelete(connection.id)}
							title="Delete Connection"
						>
							<Trash2 class="h-4 w-4 text-destructive" />
						</Button>
					</div>
				</div>
			</div>
		{:else}
			<div class="text-center py-12">
				<Database class="h-12 w-12 text-muted-foreground mx-auto mb-4" />
				<h3 class="text-lg font-medium text-foreground mb-2">No connections yet</h3>
				<p class="text-muted-foreground mb-4">Add your first PostgreSQL connection to get started</p>
				<Button onclick={() => showForm = true}>
					<Plus class="h-4 w-4 mr-2" />
					Add Connection
				</Button>
			</div>
		{/each}
	</div>
	
	<!-- Delete Confirmation Dialog -->
	{#if deleteConfirm.show}
		<div class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
			<div class="bg-card border rounded-lg p-6 max-w-md mx-4">
				<h3 class="text-lg font-semibold text-foreground mb-4">Confirm Deletion</h3>
				<p class="text-muted-foreground mb-6">
					Are you sure you want to delete this connection? This action cannot be undone.
				</p>
				<div class="flex gap-3 justify-end">
					<Button variant="outline" onclick={cancelDelete}>
						Cancel
					</Button>
					<Button variant="destructive" onclick={confirmDelete}>
						Delete Connection
					</Button>
				</div>
			</div>
		</div>
	{/if}
</div>