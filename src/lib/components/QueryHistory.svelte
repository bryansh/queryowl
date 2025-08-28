<script lang="ts">
	import { onMount } from 'svelte';
	import { History, Search, Play, Edit, Trash2, FileText, Calendar, Clock, Database, Copy, CheckCircle } from 'lucide-svelte';
	import type { DatabaseConnection } from '$lib/types/database';
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	
	let { 
		activeConnection,
		onRunQuery,
		onEditQuery
	}: { 
		activeConnection: DatabaseConnection | null;
		onRunQuery?: (sql: string) => void;
		onEditQuery?: (sql: string) => void;
	} = $props();
	
	let queryHistory: string[] = $state([]);
	let searchQuery = $state('');
	let selectedQuery = $state<string | null>(null);
	let deleteModalOpen = $state(false);
	let queryToDelete = $state<string | null>(null);
	let queryModalOpen = $state(false);
	let modalQuery = $state<string | null>(null);
	let copied = $state(false);
	
	onMount(() => {
		loadQueryHistory();
	});
	
	function loadQueryHistory() {
		const saved = localStorage.getItem('queryowl_query_history');
		if (saved) {
			try {
				queryHistory = JSON.parse(saved);
			} catch {
				queryHistory = [];
			}
		}
	}
	
	function saveQueryHistory() {
		localStorage.setItem('queryowl_query_history', JSON.stringify(queryHistory));
	}
	
	// Filter queries based on search
	let filteredQueries = $derived.by(() => {
		if (!searchQuery.trim()) {
			return queryHistory;
		}
		const search = searchQuery.toLowerCase();
		return queryHistory.filter(query => 
			query.toLowerCase().includes(search)
		);
	});
	
	function getQueryPreview(sql: string, maxLength = 80): string {
		const cleaned = sql.replace(/\s+/g, ' ').trim();
		return cleaned.length > maxLength 
			? cleaned.substring(0, maxLength) + '...'
			: cleaned;
	}
	
	function getQueryType(sql: string): string {
		const trimmed = sql.trim().toUpperCase();
		if (trimmed.startsWith('SELECT')) return 'SELECT';
		if (trimmed.startsWith('INSERT')) return 'INSERT';
		if (trimmed.startsWith('UPDATE')) return 'UPDATE';
		if (trimmed.startsWith('DELETE')) return 'DELETE';
		if (trimmed.startsWith('CREATE')) return 'CREATE';
		if (trimmed.startsWith('DROP')) return 'DROP';
		if (trimmed.startsWith('ALTER')) return 'ALTER';
		return 'OTHER';
	}
	
	function getQueryTypeColor(type: string): string {
		switch (type) {
			case 'SELECT': return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
			case 'INSERT': return 'bg-green-500/20 text-green-400 border-green-500/30';
			case 'UPDATE': return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
			case 'DELETE': return 'bg-red-500/20 text-red-400 border-red-500/30';
			case 'CREATE': return 'bg-purple-500/20 text-purple-400 border-purple-500/30';
			case 'DROP': return 'bg-orange-500/20 text-orange-400 border-orange-500/30';
			case 'ALTER': return 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30';
			default: return 'bg-surface-500/20 text-surface-400 border-surface-500/30';
		}
	}
	
	function handleRunQuery(sql) {
		if (onRunQuery && activeConnection) {
			onRunQuery(sql);
		}
	}
	
	function handleEditQuery(sql) {
		if (onEditQuery && activeConnection) {
			onEditQuery(sql);
		}
	}
	
	function handleDeleteQuery(sql) {
		queryToDelete = sql;
		deleteModalOpen = true;
	}
	
	function confirmDelete() {
		if (queryToDelete) {
			queryHistory = queryHistory.filter(q => q !== queryToDelete);
			saveQueryHistory();
		}
		deleteModalOpen = false;
		queryToDelete = null;
		selectedQuery = null;
	}
	
	function cancelDelete() {
		deleteModalOpen = false;
		queryToDelete = null;
	}
	
	function clearAllHistory() {
		queryHistory = [];
		localStorage.removeItem('queryowl_query_history');
		selectedQuery = null;
	}
	
	function selectQuery(sql) {
		modalQuery = sql;
		queryModalOpen = true;
	}
	
	function closeQueryModal() {
		queryModalOpen = false;
		modalQuery = null;
		copied = false;
	}
	
	function copyQueryToClipboard() {
		if (modalQuery) {
			navigator.clipboard.writeText(modalQuery);
			copied = true;
			setTimeout(() => copied = false, 2000);
		}
	}
</script>

<div class="p-6 pt-2 max-w-6xl mx-auto h-full flex flex-col">
	<!-- Search and Actions Bar -->
	<div class="flex items-center justify-between mb-6">
		<div class="flex items-center gap-4 flex-1">
			<div class="relative flex-1 max-w-md">
				<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-surface-400" />
				<input
					bind:value={searchQuery}
					type="text"
					placeholder="Search query history..."
					class="input pl-10 pr-4 py-2 bg-surface-100-900 border border-surface-300-600 rounded-lg w-full"
				/>
			</div>
			<div class="flex items-center gap-2 text-surface-500 text-sm">
				<History class="h-4 w-4" />
				<span>{filteredQueries.length} {filteredQueries.length === 1 ? 'query' : 'queries'}</span>
			</div>
		</div>
		
		{#if queryHistory.length > 0}
			<button
				onclick={clearAllHistory}
				class="btn btn-ghost-surface px-4 py-2 bg-surface-200-700 hover:bg-surface-300-600 border border-surface-300-600 rounded-lg transition-colors"
			>
				<Trash2 class="h-4 w-4 mr-2" />
				Clear All
			</button>
		{/if}
	</div>

	<!-- Query History List -->
	<div class="flex-1 min-h-0">
		{#if filteredQueries.length > 0}
			<div class="flex flex-col gap-2 overflow-y-auto">
				{#each filteredQueries as query, i (query)}
					<div 
						class="card p-3 cursor-pointer hover:bg-surface-200-700/50 transition-colors"
						onclick={() => selectQuery(query)}
					>
						<div class="flex items-start justify-between gap-4">
							<div class="flex-1 min-w-0">
								<div class="flex items-center gap-3 mb-1.5">
									<span class={`px-2 py-1 text-xs font-medium rounded border ${getQueryTypeColor(getQueryType(query))}`}>
										{getQueryType(query)}
									</span>
									<div class="flex items-center gap-2 text-xs text-surface-500">
										<Clock class="h-3 w-3" />
										<span>Query #{queryHistory.indexOf(query) + 1}</span>
									</div>
								</div>
								
								<div class="font-mono text-sm text-surface-300 leading-tight truncate">
									{getQueryPreview(query)}
								</div>
							</div>
							
							<div class="flex items-center gap-2 flex-shrink-0">
								<button 
									onclick={(e) => { e.stopPropagation(); handleRunQuery(query); }}
									disabled={!activeConnection}
									title="Run Query"
									class="btn btn-sm btn-filled-primary disabled:opacity-50"
								>
									<Play class="h-4 w-4" />
								</button>
								<button 
									onclick={(e) => { e.stopPropagation(); handleEditQuery(query); }}
									disabled={!activeConnection}
									title="Edit Query"
									class="btn btn-sm btn-ghost-surface"
								>
									<Edit class="h-4 w-4" />
								</button>
								<button 
									onclick={(e) => { e.stopPropagation(); handleDeleteQuery(query); }}
									title="Delete Query"
									class="btn btn-sm btn-filled-error"
								>
									<Trash2 class="h-4 w-4" />
								</button>
							</div>
						</div>
					</div>
				{/each}
			</div>
		{:else if searchQuery.trim()}
			<div class="flex-1 flex items-center justify-center text-surface-400">
				<div class="text-center">
					<Search class="h-16 w-16 mx-auto mb-4 opacity-50" />
					<p class="text-xl font-medium">No queries found</p>
					<p class="text-base mt-2 opacity-75">Try adjusting your search terms</p>
				</div>
			</div>
		{:else}
			<div class="flex-1 flex items-center justify-center text-surface-400">
				<div class="text-center">
					<History class="h-16 w-16 mx-auto mb-4 opacity-50 text-primary-500" />
					<p class="text-xl font-medium">No query history yet</p>
					<p class="text-base mt-2 opacity-75">Execute queries in the Query Editor to build your history</p>
					{#if !activeConnection}
						<div class="mt-4 flex items-center justify-center gap-2 text-surface-500">
							<Database class="h-4 w-4" />
							<span class="text-sm">Connect to a database first</span>
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>

<!-- Query View Modal -->
<Modal 
	open={queryModalOpen}
	onOpenChange={(e) => (queryModalOpen = e.open)}
	contentBase="card bg-surface-100-900 p-0 space-y-0 shadow-xl max-w-4xl mx-4 max-h-[80vh] flex flex-col"
>
	{#snippet content()}
		{#if modalQuery}
			<!-- Modal Header -->
			<div class="flex items-center justify-between p-6 border-b border-surface-300-600">
				<div class="flex items-center gap-3">
					<span class={`px-2 py-1 text-xs font-medium rounded border ${getQueryTypeColor(getQueryType(modalQuery))}`}>
						{getQueryType(modalQuery)}
					</span>
					<h3 class="text-lg font-semibold">Query Details</h3>
				</div>
				<button 
					onclick={closeQueryModal}
					class="btn btn-sm btn-ghost-surface hover:bg-surface-300-600"
					title="Close"
				>
					✕
				</button>
			</div>
			
			<!-- Query Content -->
			<div class="flex-1 p-6 overflow-y-auto">
				<div class="relative bg-surface-200-700 p-4 rounded-lg border border-surface-300-600">
					<pre class="font-mono text-sm text-surface-200 leading-relaxed whitespace-pre-wrap overflow-x-auto pr-12">{modalQuery}</pre>
					<button 
						onclick={copyQueryToClipboard}
						class="absolute top-3 right-3 btn btn-sm btn-ghost-surface hover:bg-surface-300-600 border border-surface-400-500"
						title="Copy Query"
					>
						{#if copied}
							<CheckCircle class="h-4 w-4 text-green-400" />
						{:else}
							<Copy class="h-4 w-4" />
						{/if}
					</button>
				</div>
			</div>
			
			<!-- Modal Actions -->
			<div class="flex gap-3 justify-end p-6 border-t border-surface-300-600">
				<button 
					onclick={closeQueryModal}
					class="btn btn-ghost-surface border border-surface-400-500 hover:bg-surface-300-600"
				>
					Close
				</button>
				<button 
					onclick={() => { if (modalQuery) handleDeleteQuery(modalQuery); closeQueryModal(); }}
					class="btn btn-filled-error border border-red-500 hover:border-red-400"
				>
					<Trash2 class="h-4 w-4 mr-2" />
					Delete
				</button>
				<button 
					onclick={() => { if (modalQuery) handleEditQuery(modalQuery); closeQueryModal(); }}
					disabled={!activeConnection}
					class="btn btn-ghost-surface border border-surface-400-500 hover:bg-surface-300-600 disabled:opacity-50 disabled:cursor-not-allowed"
				>
					<Edit class="h-4 w-4 mr-2" />
					Edit
				</button>
				<button 
					onclick={() => { if (modalQuery) handleRunQuery(modalQuery); closeQueryModal(); }}
					disabled={!activeConnection}
					class="btn btn-filled-primary border border-primary-500 hover:border-primary-400 disabled:opacity-50 disabled:cursor-not-allowed"
				>
					<Play class="h-4 w-4 mr-2" />
					Run Query
				</button>
			</div>
		{/if}
	{/snippet}
</Modal>

<!-- Delete Confirmation Modal -->
<Modal 
	open={deleteModalOpen}
	onOpenChange={(e) => (deleteModalOpen = e.open)}
	contentBase="card bg-surface-100-900 p-6 space-y-4 shadow-xl max-w-md mx-4"
>
	{#snippet content()}
		<h3 class="text-lg font-semibold">Delete Query</h3>
		<p class="text-surface-500">
			Are you sure you want to delete this query from your history? This action cannot be undone.
		</p>
		{#if queryToDelete}
			<div class="bg-surface-200-700 p-3 rounded-lg border border-surface-300-600">
				<div class="font-mono text-sm text-surface-300">
					{getQueryPreview(queryToDelete, 100)}
				</div>
			</div>
		{/if}
		<div class="flex gap-3 justify-end">
			<button onclick={cancelDelete} class="btn btn-ghost-surface">
				Cancel
			</button>
			<button onclick={confirmDelete} class="btn btn-filled-error">
				Delete Query
			</button>
		</div>
	{/snippet}
</Modal>