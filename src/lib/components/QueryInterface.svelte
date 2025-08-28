<script lang="ts">
	import { onMount } from 'svelte';
	import SqlEditor from '$lib/components/SqlEditor.svelte';
	import QueryDataGrid from '$lib/components/QueryDataGrid.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import type { DatabaseConnection } from '$lib/types/database';
	
	let { 
		activeConnection
	}: { 
		activeConnection: DatabaseConnection | null;
	} = $props();
	
	let sqlEditor: SqlEditor;
	let queryResults: any[] = $state([]);
	let queryError: string | null = $state(null);
	let isExecuting = $state(false);
	let executionTime: number | null = $state(null);
	let queryHistory: string[] = $state([]);
	
	onMount(() => {
		loadQueryHistory();
	});
	
	function handleEditorReady() {
		loadLastQuery();
	}
	
	function loadLastQuery() {
		if (queryHistory.length > 0 && sqlEditor) {
			const lastQuery = queryHistory[0]; // Most recent query is first
			sqlEditor.setValue(lastQuery);
		}
		// Focus the editor regardless of whether there was a last query
		if (sqlEditor) {
			sqlEditor.focus();
		}
	}
	
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
	
	// Function to load a query from history (called from main page)
	export function loadQuery(sql) {
		if (sqlEditor) {
			sqlEditor.setValue(sql);
			sqlEditor.focus();
		}
	}

	// Function to execute a query (called from main page)
	export function runQuery(sql) {
		return executeQuery(sql);
	}

	// Function to load and run a query in one call
	export async function loadAndRunQuery(sql) {
		loadQuery(sql);
		return executeQuery(sql);
	}
</script>

<div class="flex flex-col h-full">
	<!-- Main query interface split view -->
	<div class="flex-1 flex flex-col px-6">
		<!-- SQL Editor -->
		<div class="h-80 mb-6">
			<SqlEditor 
				bind:this={sqlEditor}
				height="100%"
				onExecute={executeQuery}
				onReady={handleEditorReady}
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

