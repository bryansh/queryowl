<script lang="ts">
	import { onMount } from 'svelte';
	import SqlEditor from '$lib/components/SqlEditor.svelte';
	import QueryDataGrid from '$lib/components/QueryDataGrid.svelte';
	import SchemaPanel from '$lib/components/SchemaPanel.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import type { DatabaseConnection } from '$lib/types/database';
	import { X, Plus } from 'lucide-svelte';
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
		error: string | null;
		executionTime: number | null;
		isExecuting: boolean;
		isDirty: boolean;
	}
	
	let tabCounter = 0;
	let queryHistory: string[] = $state([]);
	let editors: Record<string, SqlEditor> = {};
	let tabs: QueryTab[] = $state([]);
	let activeTabId: string = $state('');
	let databaseSchema: any = $state(null);
	
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
	
	// Load database schema when connection changes
	$effect(async () => {
		if (activeConnection) {
			try {
				databaseSchema = await invoke('get_database_schema', {
					connectionId: activeConnection.id
				});
				console.log('Loaded database schema for autocomplete:', databaseSchema);
			} catch (error) {
				console.error('Failed to load database schema:', error);
				databaseSchema = null;
			}
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
			// Execute the SQL query via Tauri backend
			const result = await invoke<any[]>('execute_query', {
				connectionId: activeConnection.id,
				sql: sql.trim()
			});
			
			tab.executionTime = Math.round(performance.now() - startTime);
			tab.results = Array.isArray(result) ? result : [result];
			
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
			const headers = Object.keys(activeTab.results[0]);
			const rows = activeTab.results.map(row => 
				headers.map(header => {
					const value = row[header];
					if (value === null) return 'NULL';
					if (typeof value === 'object') return JSON.stringify(value);
					const strValue = String(value);
					if (strValue.includes(',') || strValue.includes('"')) {
						return `"${strValue.replace(/"/g, '""')}"`;
					}
					return strValue;
				}).join(',')
			);
			
			const csvContent = [headers.join(','), ...rows].join('\n');
			const blob = new Blob([csvContent], { type: 'text/csv' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `query_results_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.csv`;
			a.click();
			URL.revokeObjectURL(url);
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
	{#if tabs.length > 0}
		<div class="flex h-full">
			<!-- Schema Panel -->
			<SchemaPanel 
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

