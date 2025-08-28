<script lang="ts">
	import { onMount } from 'svelte';
	import { Database, Plus, Trash2, TestTube, Loader2, AlertCircle, CheckCircle, Edit } from 'lucide-svelte';
	import { connections, loadConnections, saveConnection, updateConnection, deleteConnection, testConnection, connectToDatabase } from '$lib/stores/connections';
	import type { CreateConnectionRequest, UpdateConnectionRequest, TestConnectionRequest, TestConnectionResponse } from '$lib/types/database';
	import { CONNECTION_COLORS, getRandomConnectionColor } from '$lib/utils/colors';
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
		ssl: false,
		color: getRandomConnectionColor()
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
			ssl: connection.ssl || false,
			color: connection.color || getRandomConnectionColor()
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
			ssl: false,
			color: getRandomConnectionColor()
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
		<button onclick={() => showForm = !showForm} class="btn btn-primary">
			<Plus class="h-4 w-4 mr-2" />
			Add Connection
		</button>
	</div>

	{#if showForm}
		<div class="bg-card border rounded-lg p-16 mb-6 w-full h-full min-h-[80vh]">
			<div class="max-w-6xl mx-auto h-full flex flex-col">
				<h2 class="text-4xl font-bold mb-12 text-foreground text-center">
					{editingId ? 'Edit PostgreSQL Connection' : 'New PostgreSQL Connection'}
				</h2>
				<form onsubmit={handleSubmit} class="flex-1 flex flex-col justify-center space-y-16">
					<!-- First Row -->
					<div class="queryowl-form-grid">
						<div class="queryowl-field">
							<label class="queryowl-label">Connection Name</label>
							<input 
								bind:value={formData.name} 
								placeholder="My Database"
								required
								class="queryowl-input"
							/>
						</div>
						<div class="queryowl-field">
							<label class="queryowl-label">Database Name</label>
							<input 
								bind:value={formData.database} 
								placeholder="myapp_production"
								required
								class="queryowl-input"
							/>
						</div>
						<div class="queryowl-field">
							<label class="queryowl-label">Host</label>
							<input 
								bind:value={formData.host} 
								placeholder="localhost"
								required
								class="queryowl-input"
							/>
						</div>
					</div>
					
					<!-- Second Row -->
					<div class="queryowl-form-grid">
						<div class="queryowl-field">
							<label class="queryowl-label">Port</label>
							<input 
								bind:value={formData.port} 
								type="number"
								placeholder="5432"
								required
								class="queryowl-input"
							/>
						</div>
						<div class="queryowl-field">
							<label class="queryowl-label">Username</label>
							<input 
								bind:value={formData.username} 
								placeholder="postgres"
								required
								class="queryowl-input"
							/>
						</div>
						<div class="queryowl-field">
							<label class="queryowl-label">Password</label>
							<input 
								bind:value={formData.password} 
								type="password"
								placeholder={editingId ? "Enter password (required)" : "••••••••"}
								required
								class="queryowl-input"
							/>
						</div>
					</div>
				
					<!-- Color Picker -->
					<div>
						<label class="queryowl-label">Connection Color</label>
						<div class="flex flex-wrap gap-[2rem]">
							{#each CONNECTION_COLORS as color}
								<button
									type="button"
									class="w-[3rem] h-[3rem] rounded transition-all duration-200 hover:scale-105 cursor-pointer shadow-lg {formData.color === color ? 'ring-2 ring-blue-400' : ''}"
									style="background-color: {color};"
									onclick={() => formData.color = color}
									title={color}
								></button>
							{/each}
						</div>
					</div>
					
					<div class="flex items-center justify-center gap-6">
						<input 
							type="checkbox" 
							bind:checked={formData.ssl} 
							id="ssl"
							class="rounded border-input w-8 h-8"
						/>
						<label for="ssl" class="queryowl-label">Use SSL</label>
					</div>
					
					<div class="flex gap-8 pt-8 justify-center">
						<button type="submit" disabled={isLoading} class="queryowl-button">
							{#if isLoading}
								<Loader2 class="h-6 w-6 mr-4 animate-spin" />
							{/if}
							{editingId ? 'Update Connection' : 'Save Connection'}
						</button>
						<button type="button" onclick={() => { showForm = false; resetForm(); }} class="queryowl-button queryowl-button-outline">
							Cancel
						</button>
					</div>
				</form>
			</div>
		</div>
	{/if}

	{#if !showForm}
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
						<button 
							onclick={() => handleTest(connection)}
							title="Test Connection"
							class="btn btn-sm btn-outline"
						>
							<TestTube class="h-4 w-4" />
						</button>
						<button 
							onclick={() => handleConnect(connection)}
							class="btn btn-sm btn-primary"
						>
							Connect
						</button>
						<button 
							onclick={() => handleEdit(connection)}
							title="Edit Connection"
							class="btn btn-sm btn-outline"
						>
							<Edit class="h-4 w-4" />
						</button>
						<button 
							onclick={() => handleDelete(connection.id)}
							title="Delete Connection"
							class="btn btn-sm btn-outline btn-error"
						>
							<Trash2 class="h-4 w-4" />
						</button>
					</div>
				</div>
			</div>
		{:else}
			<div class="text-center py-12">
				<Database class="h-12 w-12 text-muted-foreground mx-auto mb-4" />
				<h3 class="text-lg font-medium text-foreground mb-2">No connections yet</h3>
				<p class="text-muted-foreground mb-4">Add your first PostgreSQL connection to get started</p>
				<button onclick={() => showForm = true} class="btn btn-primary">
					<Plus class="h-4 w-4 mr-2" />
					Add Connection
				</button>
			</div>
		{/each}
		</div>
	{/if}
	
	<!-- Delete Confirmation Dialog -->
	{#if deleteConfirm.show}
		<div class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
			<div class="bg-card border rounded-lg p-6 max-w-md mx-4">
				<h3 class="text-lg font-semibold text-foreground mb-4">Confirm Deletion</h3>
				<p class="text-muted-foreground mb-6">
					Are you sure you want to delete this connection? This action cannot be undone.
				</p>
				<div class="flex gap-3 justify-end">
					<button onclick={cancelDelete} class="btn btn-outline">
						Cancel
					</button>
					<button onclick={confirmDelete} class="btn btn-error">
						Delete Connection
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>