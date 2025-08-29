<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Play, Edit, Trash2, GripVertical, Plus, Search, Copy, FileCode } from 'lucide-svelte';
	import type { DatabaseConnection } from '$lib/types/database';
	
	interface SavedQuery {
		id: string;
		name: string;
		sql: string;
		description?: string;
		created_at: string;
		updated_at: string;
		connection_id?: string;
	}
	
	let { 
		activeConnection,
		onRunQuery,
		onEditQuery
	}: { 
		activeConnection: DatabaseConnection | null;
		onRunQuery: (sql: string) => void;
		onEditQuery: (sql: string) => void;
	} = $props();
	
	let savedQueries: SavedQuery[] = $state([]);
	let searchTerm = $state('');
	let draggedIndex = $state<number | null>(null);
	let dragOverIndex = $state<number | null>(null);
	let showDeleteConfirm = $state<string | null>(null);
	let copiedId = $state<string | null>(null);
	
	// Load saved queries from localStorage
	function loadSavedQueries() {
		const stored = localStorage.getItem('queryowl_saved_queries');
		if (stored) {
			try {
				savedQueries = JSON.parse(stored);
			} catch (e) {
				console.error('Failed to load saved queries:', e);
				savedQueries = [];
			}
		}
	}
	
	// Save queries to localStorage
	function saveSavedQueries() {
		localStorage.setItem('queryowl_saved_queries', JSON.stringify(savedQueries));
	}
	
	// Add a new saved query (called from QueryInterface)
	export function addSavedQuery(name: string, sql: string, description?: string) {
		const newQuery: SavedQuery = {
			id: `query_${Date.now()}`,
			name,
			sql,
			description,
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString(),
			connection_id: activeConnection?.id
		};
		
		savedQueries = [...savedQueries, newQuery];
		saveSavedQueries();
	}
	
	// Delete a saved query
	function deleteQuery(id: string) {
		savedQueries = savedQueries.filter(q => q.id !== id);
		saveSavedQueries();
		showDeleteConfirm = null;
	}
	
	// Update query order after drag and drop
	function reorderQueries(fromIndex: number, toIndex: number) {
		const queries = [...savedQueries];
		const [removed] = queries.splice(fromIndex, 1);
		queries.splice(toIndex, 0, removed);
		savedQueries = queries;
		saveSavedQueries();
	}
	
	// Drag and drop handlers
	function handleDragStart(e: DragEvent, index: number) {
		draggedIndex = index;
		if (e.dataTransfer) {
			e.dataTransfer.effectAllowed = 'move';
			e.dataTransfer.setData('text/plain', String(index));
		}
	}
	
	function handleDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		if (e.dataTransfer) {
			e.dataTransfer.dropEffect = 'move';
		}
		dragOverIndex = index;
	}
	
	function handleDragLeave() {
		dragOverIndex = null;
	}
	
	function handleDrop(e: DragEvent, index: number) {
		e.preventDefault();
		if (draggedIndex !== null && draggedIndex !== index) {
			reorderQueries(draggedIndex, index);
		}
		draggedIndex = null;
		dragOverIndex = null;
	}
	
	function handleDragEnd() {
		draggedIndex = null;
		dragOverIndex = null;
	}
	
	// Copy query to clipboard
	function copyQuery(query: SavedQuery) {
		navigator.clipboard.writeText(query.sql);
		copiedId = query.id;
		setTimeout(() => copiedId = null, 2000);
	}
	
	// Format date for display
	function formatDate(dateString: string) {
		const date = new Date(dateString);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
		
		if (diffDays === 0) {
			const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
			if (diffHours === 0) {
				const diffMinutes = Math.floor(diffMs / (1000 * 60));
				if (diffMinutes === 0) return 'Just now';
				return `${diffMinutes} minute${diffMinutes > 1 ? 's' : ''} ago`;
			}
			return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
		} else if (diffDays === 1) {
			return 'Yesterday';
		} else if (diffDays < 7) {
			return `${diffDays} days ago`;
		} else {
			return date.toLocaleDateString();
		}
	}
	
	// Filter queries based on search term
	let filteredQueries = $derived(
		savedQueries.filter(query => {
			const search = searchTerm.toLowerCase();
			return query.name.toLowerCase().includes(search) ||
				   query.sql.toLowerCase().includes(search) ||
				   (query.description && query.description.toLowerCase().includes(search));
		})
	);
	
	onMount(() => {
		loadSavedQueries();
	});
</script>

<div class="flex flex-col h-full p-6">
	<!-- Header with search -->
	<div class="flex items-center justify-between mb-6">
		<div class="flex-1 max-w-md">
			<div class="relative">
				<Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-surface-400" />
				<input
					type="text"
					bind:value={searchTerm}
					placeholder="Search saved queries..."
					class="w-full pl-10 pr-4 py-2 bg-surface-50-950 border border-surface-300-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
				/>
			</div>
		</div>
		
		<div class="text-sm text-surface-400 ml-4">
			{filteredQueries.length} {filteredQueries.length === 1 ? 'query' : 'queries'}
		</div>
	</div>
	
	<!-- Saved queries list -->
	{#if filteredQueries.length === 0}
		<div class="flex-1 flex items-center justify-center">
			<div class="text-center">
				<FileCode class="h-16 w-16 mx-auto mb-4 text-surface-400 opacity-50" />
				<h3 class="text-xl font-medium mb-2">
					{searchTerm ? 'No queries found' : 'No saved queries yet'}
				</h3>
				<p class="text-surface-400 mb-6">
					{searchTerm ? 'Try adjusting your search terms' : 'Save queries from the Query Editor to access them here'}
				</p>
				{#if !searchTerm}
					<button
						onclick={() => onEditQuery('-- Write your SQL query here\nSELECT * FROM ')}
						class="btn btn-filled-primary"
					>
						<Plus class="h-4 w-4 mr-2" />
						Go to Query Editor
					</button>
				{/if}
			</div>
		</div>
	{:else}
		<div class="flex-1 overflow-y-auto space-y-3">
			{#each filteredQueries as query, index (query.id)}
				<div
					draggable="true"
					ondragstart={(e) => handleDragStart(e, index)}
					ondragover={(e) => handleDragOver(e, index)}
					ondragleave={handleDragLeave}
					ondrop={(e) => handleDrop(e, index)}
					ondragend={handleDragEnd}
					class="group bg-surface-100-800 rounded-lg p-4 border border-surface-300-600 transition-all hover:border-primary-500/50 cursor-move
						   {dragOverIndex === index ? 'border-primary-500 bg-primary-500/10' : ''}
						   {draggedIndex === index ? 'opacity-50' : ''}"
				>
					<div class="flex items-start gap-3">
						<!-- Drag handle -->
						<div class="pt-1 opacity-0 group-hover:opacity-100 transition-opacity">
							<GripVertical class="h-5 w-5 text-surface-400" />
						</div>
						
						<!-- Query content -->
						<div class="flex-1 min-w-0">
							<div class="flex items-start justify-between mb-2">
								<div>
									<h3 class="font-semibold text-lg">{query.name}</h3>
									{#if query.description}
										<p class="text-sm text-surface-400 mt-1">{query.description}</p>
									{/if}
								</div>
								<span class="text-xs text-surface-500 ml-4">
									{formatDate(query.updated_at)}
								</span>
							</div>
							
							<!-- SQL preview -->
							<div class="bg-surface-50-950 rounded p-3 mb-3 font-mono text-sm text-surface-300 overflow-hidden">
								<pre class="whitespace-pre-wrap break-all">{query.sql.slice(0, 200)}{query.sql.length > 200 ? '...' : ''}</pre>
							</div>
							
							<!-- Action buttons -->
							<div class="flex items-center gap-2">
								<button
									onclick={() => onRunQuery(query.sql)}
									class="btn btn-ghost-primary px-3 py-1.5 text-sm"
									title="Run Query"
								>
									<Play class="h-4 w-4 mr-1" />
									Run
								</button>
								
								<button
									onclick={() => onEditQuery(query.sql)}
									class="btn btn-ghost-surface px-3 py-1.5 text-sm"
									title="Edit Query"
								>
									<Edit class="h-4 w-4 mr-1" />
									Edit
								</button>
								
								<button
									onclick={() => copyQuery(query)}
									class="btn btn-ghost-surface px-3 py-1.5 text-sm"
									title="Copy SQL"
								>
									{#if copiedId === query.id}
										<span class="text-green-500">Copied!</span>
									{:else}
										<Copy class="h-4 w-4 mr-1" />
										Copy
									{/if}
								</button>
								
								<div class="flex-1"></div>
								
								{#if showDeleteConfirm === query.id}
									<div class="flex items-center gap-2">
										<span class="text-sm text-red-400">Delete?</span>
										<button
											onclick={() => deleteQuery(query.id)}
											class="btn btn-ghost-error px-2 py-1 text-sm"
										>
											Yes
										</button>
										<button
											onclick={() => showDeleteConfirm = null}
											class="btn btn-ghost-surface px-2 py-1 text-sm"
										>
											No
										</button>
									</div>
								{:else}
									<button
										onclick={() => showDeleteConfirm = query.id}
										class="btn btn-ghost-surface px-3 py-1.5 text-sm opacity-0 group-hover:opacity-100 transition-opacity"
										title="Delete Query"
									>
										<Trash2 class="h-4 w-4 text-red-400" />
									</button>
								{/if}
							</div>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	/* Smooth drag and drop transitions */
	.group {
		transition: transform 0.2s, opacity 0.2s, border-color 0.2s;
	}
	
	.group:active {
		transform: scale(1.02);
	}
</style>