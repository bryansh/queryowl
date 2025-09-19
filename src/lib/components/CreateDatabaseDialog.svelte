<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Database, X, Loader2, AlertCircle, CheckCircle, ChevronDown, ChevronRight, Settings } from 'lucide-svelte';

	let {
		show = $bindable(false),
		onSuccess,
		onError
	}: {
		show: boolean;
		onSuccess?: (dbName: string) => void;
		onError?: (error: string) => void;
	} = $props();

	// Connection fields (to connect to PostgreSQL server)
	let host = $state('localhost');
	let port = $state(5432);
	let username = $state('');
	let password = $state('');
	let ssl = $state(false);

	// New database fields
	let newDatabaseName = $state('');
	let encoding = $state('UTF8');
	let template = $state('template0');
	let owner = $state(''); // Optional - defaults to connection user

	// UI state
	let creating = $state(false);
	let error = $state<string | null>(null);
	let success = $state(false);
	let availableDatabases = $state<string[]>([]);
	let loadingDatabases = $state(false);
	let showAdvanced = $state(false);

	// Reset form when dialog opens
	$effect(() => {
		if (show) {
			resetForm();
			loadExistingDatabases();
		}
	});

	function resetForm() {
		newDatabaseName = '';
		encoding = 'UTF8';
		template = 'template0';
		owner = '';
		error = null;
		success = false;
	}

	async function loadExistingDatabases() {
		if (!username || !password) return;

		loadingDatabases = true;
		try {
			const databases = await invoke<string[]>('list_databases', {
				host,
				port,
				username,
				password,
				ssl
			});
			availableDatabases = databases;
		} catch (err) {
			console.error('Failed to load databases:', err);
			availableDatabases = [];
		} finally {
			loadingDatabases = false;
		}
	}

	async function handleCreateDatabase() {
		if (!newDatabaseName || !username || !password) {
			error = 'Please fill in all required fields';
			return;
		}

		// Validate database name
		if (!/^[a-zA-Z0-9_]+$/.test(newDatabaseName)) {
			error = 'Database name can only contain letters, numbers, and underscores';
			return;
		}

		// Check if database already exists in our list
		if (availableDatabases.includes(newDatabaseName)) {
			error = `Database '${newDatabaseName}' already exists`;
			return;
		}

		creating = true;
		error = null;

		try {
			const result = await invoke<string>('create_database', {
				host,
				port,
				username,
				password,
				ssl,
				newDatabaseName,
				encoding: encoding || undefined,
				template: template || undefined,
				owner: owner || undefined
			});

			success = true;

			// Notify parent component
			if (onSuccess) {
				onSuccess(newDatabaseName);
			}

			// Auto-close after 2 seconds
			setTimeout(() => {
				show = false;
			}, 2000);
		} catch (err) {
			error = String(err);
			if (onError) {
				onError(String(err));
			}
		} finally {
			creating = false;
		}
	}

	function handleClose() {
		if (!creating) {
			show = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && !creating) {
			show = false;
		} else if (e.key === 'Enter' && e.ctrlKey && !creating) {
			handleCreateDatabase();
		}
	}
</script>

{#if show}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
		onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
		onkeydown={handleKeydown}
		role="dialog"
		aria-modal="true"
		aria-labelledby="create-database-title"
		tabindex="-1"
	>
		<div class="bg-surface-800 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col">
			<!-- Header -->
			<div class="p-6 border-b border-surface-600 flex items-center justify-between">
				<div class="flex items-center gap-3">
					<Database class="h-6 w-6 text-primary-400" />
					<h2 id="create-database-title" class="text-xl font-semibold">Create New Database</h2>
				</div>
				<button
					onclick={handleClose}
					disabled={creating}
					class="p-1 hover:bg-surface-700 rounded transition-colors disabled:opacity-50"
					aria-label="Close dialog"
				>
					<X class="h-5 w-5" />
				</button>
			</div>

			<!-- Content -->
			<div class="flex-1 overflow-y-auto p-6 space-y-6">
				{#if success}
					<div class="flex items-center justify-center py-12">
						<div class="text-center">
							<CheckCircle class="h-16 w-16 text-green-400 mx-auto mb-4" />
							<p class="text-xl font-medium text-green-400">Database Created Successfully!</p>
							<p class="text-sm text-surface-400 mt-2">Database '{newDatabaseName}' has been created</p>
						</div>
					</div>
				{:else}
					<!-- Server Connection Section -->
					<div>
						<h3 class="text-sm font-medium text-surface-300 mb-4 flex items-center gap-2">
							PostgreSQL Server Connection
							<span class="text-xs text-surface-500">(requires CREATE DATABASE permission)</span>
						</h3>

						<div class="grid grid-cols-2 gap-4">
							<div>
								<label for="host" class="block text-xs font-medium text-surface-400 mb-1">
									Host <span class="text-red-400">*</span>
								</label>
								<input
									id="host"
									type="text"
									bind:value={host}
									disabled={creating}
									class="w-full px-3 py-2 bg-surface-700 border border-surface-600 rounded text-sm
									       focus:border-primary-400 focus:outline-none disabled:opacity-50"
									placeholder="localhost"
								/>
							</div>

							<div>
								<label for="port" class="block text-xs font-medium text-surface-400 mb-1">
									Port <span class="text-red-400">*</span>
								</label>
								<input
									id="port"
									type="number"
									bind:value={port}
									disabled={creating}
									class="w-full px-3 py-2 bg-surface-700 border border-surface-600 rounded text-sm
									       focus:border-primary-400 focus:outline-none disabled:opacity-50"
									placeholder="5432"
								/>
							</div>

							<div>
								<label for="username" class="block text-xs font-medium text-surface-400 mb-1">
									Username <span class="text-red-400">*</span>
								</label>
								<input
									id="username"
									type="text"
									bind:value={username}
									disabled={creating}
									onblur={loadExistingDatabases}
									class="w-full px-3 py-2 bg-surface-700 border border-surface-600 rounded text-sm
									       focus:border-primary-400 focus:outline-none disabled:opacity-50"
									placeholder="postgres"
								/>
							</div>

							<div>
								<label for="password" class="block text-xs font-medium text-surface-400 mb-1">
									Password <span class="text-red-400">*</span>
								</label>
								<input
									id="password"
									type="password"
									bind:value={password}
									disabled={creating}
									onblur={loadExistingDatabases}
									class="w-full px-3 py-2 bg-surface-700 border border-surface-600 rounded text-sm
									       focus:border-primary-400 focus:outline-none disabled:opacity-50"
									placeholder="••••••••"
								/>
							</div>
						</div>

						<div class="mt-3">
							<label class="flex items-center gap-2 text-sm text-surface-400 cursor-pointer">
								<input
									type="checkbox"
									bind:checked={ssl}
									disabled={creating}
									class="rounded border-surface-600 bg-surface-700 text-primary-400
									       focus:ring-primary-400 focus:ring-offset-0 disabled:opacity-50"
								/>
								<span>Use SSL connection</span>
							</label>
						</div>
					</div>

					<!-- New Database Section -->
					<div>
						<h3 class="text-sm font-medium text-surface-300 mb-4">New Database Configuration</h3>

						<div class="space-y-4">
							<div>
								<label for="dbname" class="block text-xs font-medium text-surface-400 mb-1">
									Database Name <span class="text-red-400">*</span>
								</label>
								<input
									id="dbname"
									type="text"
									bind:value={newDatabaseName}
									disabled={creating}
									class="w-full px-3 py-2 bg-surface-700 border border-surface-600 rounded text-sm
									       focus:border-primary-400 focus:outline-none disabled:opacity-50"
									placeholder="my_database"
								/>
								<p class="text-xs text-surface-500 mt-1">
									Only letters, numbers, and underscores allowed
								</p>
							</div>

							<!-- Advanced Settings -->
							<div class="mt-6">
								<button
									type="button"
									onclick={() => showAdvanced = !showAdvanced}
									class="flex items-center gap-2 text-sm text-surface-400 hover:text-surface-300 transition-colors"
								>
									{#if showAdvanced}
										<ChevronDown class="h-4 w-4" />
									{:else}
										<ChevronRight class="h-4 w-4" />
									{/if}
									<Settings class="h-4 w-4" />
									<span>Advanced Settings</span>
									<span class="text-xs text-surface-500 ml-1">(optional)</span>
								</button>

								{#if showAdvanced}
									<div class="mt-4 p-4 bg-surface-750 rounded-lg border border-surface-600 space-y-4">
										<div class="grid grid-cols-2 gap-4">
											<div>
												<label for="encoding" class="block text-xs font-medium text-surface-400 mb-1">
													Encoding
												</label>
												<select
													id="encoding"
													bind:value={encoding}
													disabled={creating}
													class="w-full px-3 py-2 bg-surface-700 border border-surface-600 rounded text-sm
													       focus:border-primary-400 focus:outline-none disabled:opacity-50"
												>
													<option value="UTF8">UTF8 (recommended)</option>
													<option value="LATIN1">LATIN1</option>
													<option value="SQL_ASCII">SQL_ASCII</option>
													<option value="WIN1252">WIN1252</option>
												</select>
												<p class="text-xs text-surface-500 mt-1">Character encoding for the database</p>
											</div>

											<div>
												<label for="template" class="block text-xs font-medium text-surface-400 mb-1">
													Template
												</label>
												<select
													id="template"
													bind:value={template}
													disabled={creating}
													class="w-full px-3 py-2 bg-surface-700 border border-surface-600 rounded text-sm
													       focus:border-primary-400 focus:outline-none disabled:opacity-50"
												>
													<option value="template0">template0 (recommended)</option>
													<option value="template1">template1</option>
												</select>
												<p class="text-xs text-surface-500 mt-1">Template database to copy from</p>
											</div>
										</div>

										<div>
											<label for="owner" class="block text-xs font-medium text-surface-400 mb-1">
												Owner
											</label>
											<input
												id="owner"
												type="text"
												bind:value={owner}
												disabled={creating}
												class="w-full px-3 py-2 bg-surface-700 border border-surface-600 rounded text-sm
												       focus:border-primary-400 focus:outline-none disabled:opacity-50"
												placeholder={username || 'postgres'}
											/>
											<p class="text-xs text-surface-500 mt-1">Database owner (defaults to connection user)</p>
										</div>
									</div>
								{/if}
							</div>
						</div>
					</div>

					<!-- Existing Databases (for reference) -->
					{#if availableDatabases.length > 0}
						<div>
							<h3 class="text-sm font-medium text-surface-300 mb-2">Existing Databases on Server</h3>
							<div class="flex flex-wrap gap-2">
								{#each availableDatabases as db}
									<span class="px-2 py-1 bg-surface-700 text-surface-400 rounded text-xs">
										{db}
									</span>
								{/each}
							</div>
						</div>
					{:else if loadingDatabases}
						<div class="flex items-center gap-2 text-surface-500 text-sm">
							<Loader2 class="h-4 w-4 animate-spin" />
							<span>Loading existing databases...</span>
						</div>
					{/if}

					<!-- Error message -->
					{#if error}
						<div class="p-3 bg-red-900/20 border border-red-800 rounded flex items-start gap-2">
							<AlertCircle class="h-5 w-5 text-red-400 flex-shrink-0 mt-0.5" />
							<p class="text-sm text-red-400">{error}</p>
						</div>
					{/if}
				{/if}
			</div>

			<!-- Footer -->
			{#if !success}
				<div class="p-6 border-t border-surface-600 flex items-center justify-between">
					<p class="text-xs text-surface-500">
						Press Ctrl+Enter to create database
					</p>
					<div class="flex gap-3">
						<button
							onclick={handleClose}
							disabled={creating}
							class="px-4 py-2 text-sm font-medium text-surface-300 hover:text-surface-200
							       hover:bg-surface-700 rounded transition-colors disabled:opacity-50"
						>
							Cancel
						</button>
						<button
							onclick={handleCreateDatabase}
							disabled={creating || !newDatabaseName || !username || !password}
							class="px-4 py-2 text-sm font-medium bg-primary-600 hover:bg-primary-500
							       text-white rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed
							       flex items-center gap-2"
						>
							{#if creating}
								<Loader2 class="h-4 w-4 animate-spin" />
								Creating Database...
							{:else}
								<Database class="h-4 w-4" />
								Create Database
							{/if}
						</button>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	/* Additional styles if needed */
</style>