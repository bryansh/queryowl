<script lang="ts">
	import { onMount } from 'svelte';
	import { Database, Plus, Trash2, TestTube, Loader2, AlertCircle, CheckCircle, Edit } from 'lucide-svelte';
	import { connections, loadConnections, saveConnection, updateConnection, deleteConnection, testConnection, connectToDatabase } from '$lib/stores/connections';
	import type { CreateConnectionRequest, UpdateConnectionRequest, TestConnectionRequest, TestConnectionResponse } from '$lib/types/database';
	import { CONNECTION_COLORS, getRandomConnectionColor } from '$lib/utils/colors';
	import { invoke } from '@tauri-apps/api/core';
	import { Modal } from '@skeletonlabs/skeleton-svelte';

	let showForm = $state(false);
	let editingId = $state<string | null>(null);
	let isLoading = $state(false);
	let testResults = $state<Record<string, { success: boolean; error?: string }>>({});
	let deleteModalOpen = $state(false);
	let connectionToDelete = $state<string | null>(null);
	
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
		connectionToDelete = id;
		deleteModalOpen = true;
	}
	
	async function confirmDelete() {
		if (connectionToDelete) {
			try {
				await deleteConnection(connectionToDelete);
			} catch (error) {
				console.error('Failed to delete connection:', error);
			}
		}
		deleteModalOpen = false;
		connectionToDelete = null;
	}
	
	function cancelDelete() {
		deleteModalOpen = false;
		connectionToDelete = null;
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

<div class="p-6 pt-2 max-w-6xl mx-auto">
	<div class="flex items-center justify-end mb-4">
		<button class="btn btn-filled-primary" onclick={() => showForm = !showForm}>
			<Plus class="h-4 w-4 mr-2" />
			Add Connection
		</button>
	</div>

	{#if showForm}
		<div class="card p-8 mb-6" style="border-left: 5px solid {formData.color}; padding-left: calc(2rem - 5px);">
			<h2 class="text-2xl font-bold mb-6">
				{editingId ? 'Edit PostgreSQL Connection' : 'New PostgreSQL Connection'}
			</h2>
			<form onsubmit={handleSubmit} class="space-y-6">
				<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
					<label class="label">
						<span>Connection Name</span>
						<input 
							bind:value={formData.name} 
							placeholder="My Database"
							required
							class="input"
							type="text"
						/>
					</label>
					<label class="label">
						<span>Database Name</span>
						<input 
							bind:value={formData.database} 
							placeholder="myapp_production"
							required
							class="input"
							type="text"
						/>
					</label>
					<label class="label">
						<span>Host</span>
						<input 
							bind:value={formData.host} 
							placeholder="localhost"
							required
							class="input"
							type="text"
						/>
					</label>
				</div>
				
				<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
					<label class="label">
						<span>Port</span>
						<input 
							bind:value={formData.port} 
							type="number"
							placeholder="5432"
							required
							class="input"
						/>
					</label>
					<label class="label">
						<span>Username</span>
						<input 
							bind:value={formData.username} 
							placeholder="postgres"
							required
							class="input"
							type="text"
						/>
					</label>
					<label class="label">
						<span>Password</span>
						<input 
							bind:value={formData.password} 
							type="password"
							placeholder={editingId ? "Enter password (required)" : "••••••••"}
							required
							class="input"
						/>
					</label>
				</div>
			
				<div class="space-y-4">
					<label class="label">
						<span>Connection Color</span>
						<div class="flex gap-2 mt-2">
							{#each CONNECTION_COLORS as color}
								<button
									type="button"
									class="w-10 h-10 rounded-lg transition-all duration-200 hover:scale-110 {formData.color === color ? 'ring-2 ring-primary-500' : ''}"
									style="background-color: {color};"
									onclick={() => formData.color = color}
									title={color}
								></button>
							{/each}
						</div>
					</label>
					
					<label class="flex items-center space-x-2">
						<input 
							type="checkbox" 
							bind:checked={formData.ssl} 
							class="checkbox"
						/>
						<span>Use SSL</span>
					</label>
				</div>
				
				<div class="flex justify-between pt-4">
					<button type="button" onclick={() => { showForm = false; resetForm(); }} class="btn btn-ghost-surface px-6 py-2 bg-surface-200-700 hover:bg-surface-300-600 border border-surface-300-600 rounded-lg transition-colors">
						Cancel
					</button>
					<button type="submit" disabled={isLoading} class="btn btn-filled-primary px-6 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors disabled:opacity-50">
						{#if isLoading}
							<Loader2 class="h-4 w-4 mr-2 animate-spin" />
						{/if}
						{editingId ? 'Update Connection' : 'Save Connection'}
					</button>
				</div>
			</form>
		</div>
	{/if}

	{#if !showForm}
		<div class="space-y-4">
			{#each $connections as connection (connection.id)}
				<div 
					class="card p-4 cursor-pointer hover:bg-surface-200-700/50 transition-colors" 
					style="border-left: 5px solid {connection.color || '#14b8a6'}; padding-left: calc(1rem - 5px);"
					onclick={() => handleConnect(connection)}
				>
					<div class="flex items-center justify-between">
						<div class="flex-1">
							<div class="flex items-center gap-3 mb-2">
								<Database class="h-6 w-6 text-surface-500" />
								<h3 class="text-lg font-medium">{connection.name}</h3>
							</div>
							<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-2 text-sm text-surface-500">
								<div>Host: {connection.host}:{connection.port}</div>
								<div>Database: {connection.database}</div>
								<div>User: {connection.username}</div>
								<div>Last connected: {formatLastConnected(connection.last_connected)}</div>
							</div>
							{#if testResults[connection.id]}
								<div class="mt-2 flex items-center gap-2">
									{#if testResults[connection.id].success}
										<CheckCircle class="h-4 w-4 text-success-500" />
										<span class="text-sm text-success-600">Connection successful</span>
									{:else}
										<AlertCircle class="h-4 w-4 text-error-500" />
										<span class="text-sm text-error-600">
											{testResults[connection.id].error || 'Connection failed'}
										</span>
									{/if}
								</div>
							{/if}
						</div>
						<div class="flex items-center gap-2">
							<button 
								onclick={(e) => { e.stopPropagation(); handleTest(connection); }}
								title="Test Connection"
								class="btn btn-sm btn-ghost-surface"
							>
								<TestTube class="h-5 w-5" />
							</button>
							<button 
								onclick={(e) => { e.stopPropagation(); handleConnect(connection); }}
								class="btn btn-sm btn-filled-primary"
							>
								Connect
							</button>
							<button 
								onclick={(e) => { e.stopPropagation(); handleEdit(connection); }}
								title="Edit Connection"
								class="btn btn-sm btn-ghost-surface"
							>
								<Edit class="h-5 w-5" />
							</button>
							<button 
								onclick={(e) => { e.stopPropagation(); handleDelete(connection.id); }}
								title="Delete Connection"
								class="btn btn-sm btn-filled-error"
							>
								<Trash2 class="h-5 w-5" />
							</button>
						</div>
					</div>
				</div>
			{:else}
				<div class="card text-center py-12">
					<Database class="h-16 w-16 text-surface-500 mx-auto mb-4" />
					<h3 class="text-xl font-medium mb-2">No connections yet</h3>
					<p class="text-base text-surface-500 mb-4">Add your first PostgreSQL connection to get started</p>
					<button onclick={() => showForm = true} class="btn btn-filled-primary">
						<Plus class="h-4 w-4 mr-2" />
						Add Connection
					</button>
				</div>
			{/each}
		</div>
	{/if}
	
	<!-- Delete Confirmation Modal -->
	<Modal 
		open={deleteModalOpen}
		onOpenChange={(e) => (deleteModalOpen = e.open)}
		contentBase="card bg-surface-100-900 p-6 space-y-4 shadow-xl max-w-md mx-4"
	>
		{#snippet content()}
			<h3 class="text-lg font-semibold">Confirm Deletion</h3>
			<p class="text-surface-500">
				Are you sure you want to delete this connection? This action cannot be undone.
			</p>
			<div class="flex gap-3 justify-end">
				<button onclick={cancelDelete} class="btn btn-ghost-surface">
					Cancel
				</button>
				<button onclick={confirmDelete} class="btn btn-filled-error">
					Delete Connection
				</button>
			</div>
		{/snippet}
	</Modal>
</div>