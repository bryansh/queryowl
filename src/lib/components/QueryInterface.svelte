<script lang="ts">
	import { onMount } from 'svelte';
	import SqlEditor from '$lib/components/SqlEditor.svelte';
	import QueryDataGrid from '$lib/components/QueryDataGrid.svelte';
	import SchemaPanel from '$lib/components/SchemaPanel.svelte';
	import ExportDialog from '$lib/components/ExportDialog.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import type { DatabaseConnection } from '$lib/types/database';
	import { X, Plus, FileDown } from 'lucide-svelte';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	
	let { 
		activeConnection,
		onResultsChange
	}: { 
		activeConnection: DatabaseConnection | null;
		onResultsChange?: (results: any[], error: string | null, executionTime: number | null, isExecuting: boolean) => void;
	} = $props();
	
	interface QueryTab {
		id: string;
		title: string;
		sql: string;
		results: any[];
		metadata?: any;
		error: string | null;
		executionTime: number | null;
		isExecuting: boolean;
		isDirty: boolean;
	}
	
	let tabCounter = 0;
	let queryHistory: string[] = $state([]);
	let editors: Record<string, SqlEditor> = $state({});
	let tabs: QueryTab[] = $state([]);
	let activeTabId: string = $state('');
	let databaseSchema: any = $state(null);
	let schemaPanel = $state<SchemaPanel>();
	let showExportDialog = $state(false);
	
	// Initialize tabs from storage or create default
	function initializeTabs() {
		const saved = localStorage.getItem('queryowl_tabs');
		console.log('Loading tabs from storage:', saved);
		if (saved) {
			try {
				const tabsData = JSON.parse(saved);
				console.log('Parsed tabs data:', tabsData);
				if (tabsData.tabs && tabsData.tabs.length > 0) {
					tabs = tabsData.tabs;
					activeTabId = tabsData.activeTabId;
					tabCounter = tabsData.tabCounter || tabs.length;
					console.log('Successfully loaded tabs:', tabs.length, 'active:', activeTabId);
					return; // Exit early if we loaded successfully
				}
			} catch (e) {
				console.error('Failed to parse saved tabs:', e);
				// Fall through to create default tab
			}
		}
		
		console.log('Creating default tab');
		// Create default tab if no saved state or loading failed
		tabCounter++;
		tabs = [{
			id: `tab${tabCounter}`,
			title: `Query ${tabCounter}`,
			sql: '-- Write your SQL query here\nSELECT * FROM ',
			results: [],
			error: null,
			executionTime: null,
			isExecuting: false,
			isDirty: false
		}];
		activeTabId = `tab${tabCounter}`;
		saveTabsToStorage();
	}
	
	// Initialize tabs immediately
	initializeTabs();
	
	// Function to refresh database schema
	async function refreshDatabaseSchema() {
		if (activeConnection) {
			try {
				databaseSchema = await invoke('get_database_schema', {
					connectionId: activeConnection.id
				});
				console.log('Refreshed database schema:', databaseSchema);
			} catch (error) {
				console.error('Failed to refresh database schema:', error);
			}
		}
	}

	// Check if a query affects database schema and was successful
	function isSchemaAffectingQuery(sql: string, results: any[]): boolean {
		// Only refresh schema if the query was successful (has success status)
		const isSuccessful = results.length === 1 && results[0].status === 'success';
		if (!isSuccessful) return false;

		const sqlUpper = sql.trim().toUpperCase();
		
		// DDL queries that affect schema structure
		return sqlUpper.startsWith('CREATE TABLE') ||
			   sqlUpper.startsWith('CREATE VIEW') ||
			   sqlUpper.startsWith('CREATE MATERIALIZED VIEW') ||
			   sqlUpper.startsWith('CREATE INDEX') ||
			   sqlUpper.startsWith('CREATE UNIQUE INDEX') ||
			   sqlUpper.startsWith('CREATE FUNCTION') ||
			   sqlUpper.startsWith('CREATE PROCEDURE') ||
			   sqlUpper.startsWith('CREATE SEQUENCE') ||
			   sqlUpper.startsWith('CREATE TYPE') ||
			   sqlUpper.startsWith('CREATE SCHEMA') ||
			   sqlUpper.startsWith('DROP TABLE') ||
			   sqlUpper.startsWith('DROP VIEW') ||
			   sqlUpper.startsWith('DROP MATERIALIZED VIEW') ||
			   sqlUpper.startsWith('DROP INDEX') ||
			   sqlUpper.startsWith('DROP FUNCTION') ||
			   sqlUpper.startsWith('DROP PROCEDURE') ||
			   sqlUpper.startsWith('DROP SEQUENCE') ||
			   sqlUpper.startsWith('DROP TYPE') ||
			   sqlUpper.startsWith('DROP SCHEMA') ||
			   sqlUpper.startsWith('ALTER TABLE') ||
			   sqlUpper.startsWith('ALTER VIEW') ||
			   sqlUpper.startsWith('ALTER FUNCTION') ||
			   sqlUpper.startsWith('ALTER SEQUENCE') ||
			   sqlUpper.startsWith('ALTER TYPE') ||
			   sqlUpper.startsWith('ALTER SCHEMA') ||
			   sqlUpper.startsWith('RENAME TABLE') ||
			   sqlUpper.startsWith('TRUNCATE TABLE'); // TRUNCATE doesn't change structure but might affect row counts
	}

	// Load database schema when connection changes
	$effect(async () => {
		if (activeConnection) {
			await refreshDatabaseSchema();
		} else {
			databaseSchema = null;
		}
	});
	
	// Watch for tab changes to focus editor
	$effect(() => {
		if (activeTabId) {
			// Focus the editor after a tick
			requestAnimationFrame(() => {
				const editor = editors[activeTabId];
				if (editor) {
					editor.focus();
				}
			});
		}
		});

	// Notify parent when active tab changes or results are available
	$effect(() => {
		if (activeTab && onResultsChange) {
			onResultsChange(
				activeTab.results || [],
				activeTab.error,
				activeTab.executionTime,
				activeTab.isExecuting
			);
		}
	});

	onMount(() => {
		loadQueryHistory();
		
		// Add keyboard shortcut for new tab
		const handleKeyDown = (e: KeyboardEvent) => {
			if ((e.metaKey || e.ctrlKey) && e.key === 't') {
				e.preventDefault();
				createNewTab();
			}
		};
		
		document.addEventListener('keydown', handleKeyDown);
		return () => document.removeEventListener('keydown', handleKeyDown);
	});
	
	function createNewTab(sql = '-- Write your SQL query here\nSELECT * FROM ', title = null) {
		tabCounter++;
		const id = `tab${tabCounter}`; // Simplified ID for Skeleton tabs
		const newTab: QueryTab = {
			id,
			title: title || `Query ${tabCounter}`,
			sql,
			results: [],
			error: null,
			executionTime: null,
			isExecuting: false,
			isDirty: false
		};
		tabs = [...tabs, newTab];
		activeTabId = id;
		return id;
	}
	
	function closeTab(id: string) {
		const index = tabs.findIndex(t => t.id === id);
		if (index === -1) return;
		
		// Remove the tab and its editor
		tabs = tabs.filter(t => t.id !== id);
		delete editors[id];
		
		// If this was the active tab, activate another
		if (activeTabId === id) {
			if (tabs.length > 0) {
				// Activate the tab to the left, or the first remaining tab
				const newIndex = Math.max(0, Math.min(index - 1, tabs.length - 1));
				activeTabId = tabs[newIndex].id;
			} else {
				// Create a new tab if all were closed
				createNewTab();
			}
		}
	}
	
	function activateTab(id: string) {
		activeTabId = id;
	}
	
	function handleSqlChange(sql: string) {
		if (activeTabId) {
			const tab = tabs.find(t => t.id === activeTabId);
			if (tab) {
				tab.sql = sql;
				tab.isDirty = true;
				tabs = tabs; // Trigger reactivity
				saveTabsToStorage();
			}
		}
	}
	
	function handleEditorReady(tabId: string) {
		// Focus editor when ready if it's the active tab
		if (tabId === activeTabId) {
			const editor = editors[tabId];
			if (editor) {
				editor.focus();
			}
		}
	}
	
	async function executeQuery(tabId: string, sql: string) {
		if (!activeConnection) {
			const tab = tabs.find(t => t.id === tabId);
			if (tab) {
				tab.error = 'No active database connection';
				tabs = tabs;
			}
			return;
		}
		
		const tab = tabs.find(t => t.id === tabId);
		if (!tab) return;
		
		tab.isExecuting = true;
		tab.error = null;
		tab.results = [];
		tab.executionTime = null;
		tab.isDirty = false;
		tabs = tabs; // Trigger reactivity
		
		// Notify parent about execution state
		if (onResultsChange && tabId === activeTabId) {
			onResultsChange([], null, null, true);
		}
		
		const startTime = performance.now();
		
		try {
			// Execute the SQL query via Tauri backend with default limit
			const response = await invoke<any>('execute_query', {
				connectionId: activeConnection.id,
				sql: sql.trim(),
				limit: 1000 // Default limit
			});

			tab.executionTime = Math.round(performance.now() - startTime);

			// Handle new response structure
			if (response.results) {
				tab.results = response.results;
				tab.metadata = response.metadata; // Store metadata for pagination info
			} else {
				// Fallback for old format
				tab.results = Array.isArray(response) ? response : [response];
			}
			
			// Check if this was a successful DDL query that affects schema
			const shouldRefreshSchema = isSchemaAffectingQuery(sql.trim(), tab.results);
			if (shouldRefreshSchema) {
				console.log('DDL query detected, refreshing schema...');
				await refreshDatabaseSchema();
				// Also refresh the SchemaPanel
				if (schemaPanel && schemaPanel.refreshSchema) {
					await schemaPanel.refreshSchema();
				}
			}
			
			// Add to query history
			addToHistory(sql);
			
		} catch (error) {
			tab.error = String(error);
			tab.executionTime = Math.round(performance.now() - startTime);
		} finally {
			tab.isExecuting = false;
			tabs = tabs; // Trigger reactivity
			saveTabsToStorage();
			
			// Notify parent about results
			if (onResultsChange && tabId === activeTabId) {
				onResultsChange(tab.results, tab.error, tab.executionTime, false);
			}
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
	
	function saveTabsToStorage() {
		const tabsData = {
			tabs,
			activeTabId,
			tabCounter
		};
		console.log('Saving tabs to storage:', tabsData);
		localStorage.setItem('queryowl_tabs', JSON.stringify(tabsData));
	}
	
	// Function to load a query from history (called from main page)
	export function loadQuery(sql) {
		// Create a new tab with the query or use current active tab if empty
		const activeTab = tabs.find(t => t.id === activeTabId);
		if (activeTab && activeTab.sql.trim() === '-- Write your SQL query here\nSELECT * FROM ' && !activeTab.isDirty) {
			// Use existing empty tab
			activeTab.sql = sql;
			const editor = editors[activeTabId];
			if (editor) {
				editor.setValue(sql);
				editor.focus();
			}
		} else {
			// Create new tab
			const newTabId = createNewTab(sql, `Query ${tabCounter}`);
			// Wait for editor to be ready
			requestAnimationFrame(() => {
				const editor = editors[newTabId];
				if (editor) {
					editor.focus();
				}
			});
		}
	}

	// Function to execute a query (called from main page)
	export function runQuery(sql) {
		if (activeTabId) {
			return executeQuery(activeTabId, sql);
		}
	}

	// Function to load and run a query in one call
	export async function loadAndRunQuery(sql) {
		loadQuery(sql);
		// Wait a tick for the editor to update
		await new Promise(resolve => requestAnimationFrame(resolve));
		if (activeTabId) {
			return executeQuery(activeTabId, sql);
		}
	}
	
	// Get the active tab
	let activeTab = $derived(tabs.find(t => t.id === activeTabId));
	
	// Export functions for status bar
	export function copyResults() {
		if (activeTab && activeTab.results && activeTab.results.length > 0) {
			const headers = Object.keys(activeTab.results[0]);
			const rows = activeTab.results.map(row => 
				headers.map(header => {
					const value = row[header];
					if (value === null) return 'NULL';
					if (typeof value === 'object') return JSON.stringify(value);
					return String(value);
				}).join('\t')
			);
			
			const tsvContent = [headers.join('\t'), ...rows].join('\n');
			navigator.clipboard.writeText(tsvContent);
		}
	}
	
	export function exportResults() {
		if (activeTab && activeTab.results && activeTab.results.length > 0) {
			// Skip if this is a DDL/DML success message
			if (activeTab.results.length === 1 && activeTab.results[0].status === 'success') {
				return;
			}
			
			// Show export dialog
			showExportDialog = true;
		}
	}
	
	// Function to insert text into the active tab's editor
	function insertIntoActiveTab(text: string) {
		if (activeTab && editors[activeTab.id]) {
			const editor = editors[activeTab.id];
			if (editor) {
				// Insert text at current cursor position
				const currentValue = editor.getValue();
				const newValue = currentValue + (currentValue.endsWith(' ') || currentValue.endsWith('\n') ? '' : ' ') + text;
				editor.setValue(newValue);
				editor.focus();
			}
		}
	}
	
	// Handle saving a query
	function handleSaveQuery(queryData: { name: string; sql: string; description?: string }) {
		// Save to localStorage via SavedQueries structure
		const savedQueries = JSON.parse(localStorage.getItem('queryowl_saved_queries') || '[]');
		const newQuery = {
			id: `query_${Date.now()}`,
			name: queryData.name,
			sql: queryData.sql,
			description: queryData.description,
			created_at: new Date().toISOString(),
			updated_at: new Date().toISOString(),
			connection_id: activeConnection?.id
		};
		savedQueries.push(newQuery);
		localStorage.setItem('queryowl_saved_queries', JSON.stringify(savedQueries));
		
		// Show success notification (could be enhanced)
		alert(`Query "${queryData.name}" saved successfully!`);
	}
</script>

<div class="flex flex-col h-full">
	{#if activeConnection && tabs.length > 0}
		<div class="flex h-full">
			<!-- Schema Panel -->
			<SchemaPanel 
				bind:this={schemaPanel}
				activeConnection={activeConnection} 
				onTableSelect={(name) => insertIntoActiveTab(name)}
			/>
			
			<!-- Main Query Area -->
			<div class="flex-1 flex flex-col min-w-0">
				<Tabs value={activeTabId} onValueChange={(e) => (activeTabId = e.value)}>
					{#snippet list()}
						<div class="flex items-center border-b border-surface-300-600 bg-surface-50-950">
							<div class="flex-1 flex items-center overflow-x-auto">
								{#each tabs as tab (tab.id)}
									<Tabs.Control value={tab.id} class="group">
										<div class="flex items-center gap-2 px-2">
											<span class="text-sm font-medium">
												{tab.title}
												{#if tab.isDirty}
													<span class="ml-1 text-yellow-500">•</span>
												{/if}
											</span>
											{#if tabs.length > 1}
												<button
													onclick={(e) => { e.stopPropagation(); closeTab(tab.id); }}
													class="hover:bg-surface-300-700 rounded p-1 transition-all text-surface-400 hover:text-surface-200"
													title="Close Tab"
												>
													<X class="h-3 w-3" />
												</button>
											{/if}
										</div>
									</Tabs.Control>
								{/each}
							</div>
							<button
								onclick={() => createNewTab()}
								class="ml-2 mr-4 p-2 hover:bg-surface-100-800 rounded transition-colors text-surface-400 hover:text-surface-200"
								title="New Tab (Ctrl+T)"
							>
								<Plus class="h-4 w-4" />
							</button>
						</div>
					{/snippet}
					
					{#snippet content()}
						{#each tabs as tab (tab.id)}
							<Tabs.Panel value={tab.id}>
								<div class="flex flex-col h-full px-6">
									<!-- SQL Editor - 50% of available space -->
									<div class="flex-1 mb-6 mt-4 min-h-0">
										<SqlEditor 
											bind:this={editors[tab.id]}
											bind:value={tab.sql}
											height="100%"
											onExecute={(sql) => executeQuery(tab.id, sql)}
											onReady={() => handleEditorReady(tab.id)}
											isExecuting={tab.isExecuting}
											onSave={handleSaveQuery}
											schema={databaseSchema}
										/>
									</div>

									<!-- Query Results - 50% of available space -->
									<div class="flex-1 min-h-0">
										<QueryDataGrid 
											data={tab.results}
											metadata={tab.metadata}
											error={tab.error}
											executionTime={tab.executionTime}
											maxHeight="100%"
										/>
									</div>
								</div>
							</Tabs.Panel>
						{/each}
					{/snippet}
				</Tabs>
			</div>
		</div>
	{:else if !activeConnection}
		<div class="flex items-center justify-center h-full text-surface-500">
			<div class="text-center">
				<p class="text-lg">Connecting to database...</p>
			</div>
		</div>
	{/if}
</div>

<style>
	/* Custom styles for Skeleton tabs */
	:global(.tab-list) {
		border-bottom: 1px solid rgb(71 85 105);
	}
	
	:global([data-sveltekit-preload-data="hover"]) {
		color: inherit;
	}
</style>

<!-- Export Dialog -->
{#if activeTab && showExportDialog}
	<ExportDialog 
		bind:show={showExportDialog}
		sql={activeTab.sql}
		results={activeTab.results}
		metadata={activeTab.metadata}
		connectionId={activeConnection?.id || ''}
		onClose={() => showExportDialog = false}
	/>
{/if}