<script lang="ts">
	import { onMount } from 'svelte';
	import SqlEditor from '$lib/components/SqlEditor.svelte';
	import QueryDataGrid from '$lib/components/QueryDataGrid.svelte';
	import { Database, History, Save, FileText } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import type { DatabaseConnection } from '$lib/types/database';
	
	let { activeConnection }: { activeConnection: DatabaseConnection | null } = $props();
	
	let sqlEditor: SqlEditor;
	let queryResults: any[] = $state([]);
	let queryError: string | null = $state(null);
	let isExecuting = $state(false);
	let executionTime: number | null = $state(null);
	let queryHistory: string[] = $state([]);
	let showHistory = $state(false);
	
	onMount(() => {
		loadQueryHistory();
	});
	
	async function executeQuery(sql: string) {
		if (!activeConnection) {
			queryError = 'No active database connection';
			return;
		}
		
		isExecuting = true;
		queryError = null;
		queryResults = [];
		executionTime = null;
		
		const startTime = performance.now();
		
		try {
			// Execute the SQL query via Tauri backend
			const result = await invoke<any[]>('execute_query', {
				connectionId: activeConnection.id,
				sql: sql.trim()
			});
			
			executionTime = Math.round(performance.now() - startTime);
			queryResults = Array.isArray(result) ? result : [result];
			
			// Add to query history
			addToHistory(sql);
			
		} catch (error) {
			queryError = String(error);
			executionTime = Math.round(performance.now() - startTime);
		} finally {
			isExecuting = false;
		}
	}
	
	function addToHistory(sql: string) {
		const trimmedSql = sql.trim();
		if (trimmedSql && !queryHistory.includes(trimmedSql)) {
			queryHistory = [trimmedSql, ...queryHistory.slice(0, 19)]; // Keep last 20 queries
			saveQueryHistory();
		}
	}
	
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
	
	function loadHistoryQuery(sql: string) {
		sqlEditor?.setValue(sql);
		showHistory = false;
	}
	
	function clearHistory() {
		queryHistory = [];
		localStorage.removeItem('queryowl_query_history');
	}
	
	function getQueryPreview(sql: string, maxLength = 60): string {
		const cleaned = sql.replace(/\s+/g, ' ').trim();
		return cleaned.length > maxLength 
			? cleaned.substring(0, maxLength) + '...'
			: cleaned;
	}
</script>

<div class="flex flex-col h-full">
	<!-- Query editor toolbar -->
	<div class="card flex items-center justify-between px-4 py-3 mb-2">
		<div class="flex items-center gap-3">
			<button 
				onclick={() => showHistory = !showHistory}
				class="btn btn-ghost-surface btn-sm px-3 py-2"
			>
				<History class="h-4 w-4 mr-2" />
				History
			</button>
		</div>
		<div class="text-xs text-surface-500">
			Press <kbd class="px-2 py-1 bg-surface-200-700 text-surface-900-100 rounded text-xs">Cmd+Enter</kbd> to execute
		</div>
	</div>

	<!-- Query History Dropdown -->
	{#if showHistory}
		<div class="card mx-4 mb-4 shadow-xl">
			<div class="flex items-center justify-between p-4 border-b border-surface-300-600">
				<h3 class="font-medium">Query History</h3>
				<button onclick={clearHistory} class="btn btn-ghost-surface btn-sm">
					Clear
				</button>
			</div>
			{#if queryHistory.length > 0}
				<div class="p-2">
					<div class="space-y-1 max-h-64 overflow-y-auto">
						{#each queryHistory as query, i}
							<button
								class="w-full text-left p-3 text-sm bg-surface-200-700 hover:bg-surface-300-600 rounded transition-colors font-mono"
								onclick={() => loadHistoryQuery(query)}
							>
								{getQueryPreview(query)}
							</button>
						{/each}
					</div>
				</div>
			{:else}
				<div class="text-center py-8 text-surface-500">
					<FileText class="h-8 w-8 mx-auto mb-2 opacity-50" />
					<p>No query history yet</p>
				</div>
			{/if}
		</div>
	{/if}

	<!-- Main query interface split view -->
	<div class="flex-1 flex flex-col">
		<!-- SQL Editor -->
		<div class="h-80 mb-4">
			<SqlEditor 
				bind:this={sqlEditor}
				height="100%"
				onExecute={executeQuery}
				{isExecuting}
			/>
		</div>

		<!-- Query Results -->
		<div class="flex-1 min-h-0">
			<QueryDataGrid 
				data={queryResults}
				error={queryError}
				{executionTime}
				maxHeight="100%"
			/>
		</div>
	</div>
</div>

