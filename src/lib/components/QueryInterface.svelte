<script lang="ts">
	import { onMount } from 'svelte';
	import SqlEditor from '$lib/components/SqlEditor.svelte';
	import QueryResults from '$lib/components/QueryResults.svelte';
	import { Button } from '$lib/components/ui/button';
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

<div class="query-interface flex flex-col h-full gap-4">
	<!-- Connection Status -->
	<div class="connection-status flex items-center justify-between p-3 bg-muted/30 rounded-lg border">
		{#if activeConnection}
			<div class="flex items-center gap-3">
				<Database class="h-5 w-5 text-green-500" />
				<div>
					<div class="font-medium">{activeConnection.name}</div>
					<div class="text-sm text-muted-foreground">
						{activeConnection.username}@{activeConnection.host}:{activeConnection.port}/{activeConnection.database}
					</div>
				</div>
			</div>
		{:else}
			<div class="flex items-center gap-3">
				<Database class="h-5 w-5 text-muted-foreground" />
				<div class="text-muted-foreground">No active connection</div>
			</div>
		{/if}
		
		<div class="flex items-center gap-2">
			<Button 
				size="sm" 
				variant="outline"
				onclick={() => showHistory = !showHistory}
			>
				<History class="h-4 w-4 mr-2" />
				History
			</Button>
		</div>
	</div>

	<!-- Query History Dropdown -->
	{#if showHistory}
		<div class="query-history bg-card border rounded-lg p-3">
			<div class="flex items-center justify-between mb-3">
				<h3 class="font-medium">Query History</h3>
				<Button size="sm" variant="ghost" onclick={clearHistory}>
					Clear
				</Button>
			</div>
			{#if queryHistory.length > 0}
				<div class="space-y-2 max-h-64 overflow-y-auto">
					{#each queryHistory as query, i}
						<button
							class="w-full text-left p-2 text-sm bg-muted/50 hover:bg-muted rounded border transition-colors font-mono"
							onclick={() => loadHistoryQuery(query)}
						>
							{getQueryPreview(query)}
						</button>
					{/each}
				</div>
			{:else}
				<div class="text-center py-4 text-muted-foreground">
					<FileText class="h-8 w-8 mx-auto mb-2 opacity-50" />
					<p>No query history yet</p>
				</div>
			{/if}
		</div>
	{/if}

	<!-- SQL Editor -->
	<div class="sql-editor flex-1">
		<SqlEditor 
			bind:this={sqlEditor}
			height="300px"
			onExecute={executeQuery}
			{isExecuting}
		/>
	</div>

	<!-- Query Results -->
	<div class="query-results flex-1">
		<QueryResults 
			results={queryResults}
			error={queryError}
			{executionTime}
		/>
	</div>
</div>

<style>
	.query-interface {
		height: 100%;
		min-height: 600px;
	}
	
	.sql-editor, .query-results {
		min-height: 200px;
	}
</style>