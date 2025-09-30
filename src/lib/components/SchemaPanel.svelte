<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Database, Table, Eye, ChevronDown, ChevronRight, Columns, Hash, Type, Key, Zap, Settings, Link, Shield, FileText, Package, Folder, Layers, Copy, Check } from 'lucide-svelte';
	import type { DatabaseConnection } from '$lib/types/database';
	
	let { 
		activeConnection,
		onTableSelect
	}: { 
		activeConnection: DatabaseConnection | null;
		onTableSelect?: (tableName: string) => void;
	} = $props();
	
	interface SchemaTable {
		table_name: string;
		table_schema: string;
		table_type: string;
		column_count: number;
	}
	
	interface SchemaColumn {
		column_name: string;
		data_type: string;
		is_nullable: string;
		column_default: string | null;
		is_primary_key: boolean;
	}

	interface SchemaIndex {
		index_name: string;
		table_name: string;
		column_names: string[];
		is_unique: boolean;
		is_primary: boolean;
		index_type: string;
	}

	interface SchemaFunction {
		function_name: string;
		schema_name: string;
		return_type: string;
		parameters: string[];
		function_type: string;
	}

	interface SchemaTrigger {
		trigger_name: string;
		table_name: string;
		event_manipulation: string;
		action_timing: string;
		action_statement: string;
	}

	interface SchemaSequence {
		sequence_name: string;
		data_type: string;
		start_value: string;
		increment: string;
		max_value: string;
		min_value: string;
	}

	interface SchemaForeignKey {
		constraint_name: string;
		table_name: string;
		column_name: string;
		foreign_table_name: string;
		foreign_column_name: string;
		update_rule: string;
		delete_rule: string;
	}

	interface SchemaConstraint {
		constraint_name: string;
		table_name: string;
		constraint_type: string;
		column_names: string[];
		check_clause: string | null;
	}

	interface SchemaEnum {
		type_name: string;
		enum_values: string[];
	}

	interface SchemaSchema {
		schema_name: string;
		owner: string;
	}
	
	interface DatabaseSchema {
		tables: SchemaTable[];
		views: SchemaTable[];
		materialized_views: SchemaTable[];
		indexes: SchemaIndex[];
		functions: SchemaFunction[];
		triggers: SchemaTrigger[];
		sequences: SchemaSequence[];
		foreign_keys: SchemaForeignKey[];
		constraints: SchemaConstraint[];
		enums: SchemaEnum[];
		schemas: SchemaSchema[];
	}
	
	let schema: DatabaseSchema | null = $state(null);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let expandedSchemas = $state<Set<string>>(new Set(['public'])); // Expand 'public' by default
	let expandedTables = $state<Set<string>>(new Set());
	let tableColumns = $state<Record<string, SchemaColumn[]>>({});
	let loadingColumns = $state<Set<string>>(new Set());
	let copiedTables = $state<Set<string>>(new Set());

	// Section collapse states - all expanded by default
	let expandedSections = $state<Set<string>>(new Set([
		'tables', 'views', 'materialized_views', 'indexes', 
		'functions', 'triggers', 'sequences', 'foreign_keys', 
		'constraints', 'enums', 'schemas'
	]));
	
	onMount(() => {
		if (activeConnection) {
			loadSchema();
		}
	});
	
	// Reload schema when connection changes
	$effect(() => {
		if (activeConnection) {
			loadSchema();
		} else {
			schema = null;
		}
	});
	
	async function loadSchema() {
		if (!activeConnection) return;
		
		loading = true;
		error = null;
		
		try {
			const result = await invoke<DatabaseSchema>('get_database_schema', {
				connectionId: activeConnection.id
			});
			schema = result;
		} catch (err) {
			error = String(err);
			console.error('Failed to load schema:', err);
		} finally {
			loading = false;
		}
	}

	// Export function to allow external refresh
	export async function refreshSchema() {
		console.log('SchemaPanel: Refreshing schema...');
		await loadSchema();
	}
	
	async function toggleTable(tableName: string) {
		if (expandedTables.has(tableName)) {
			expandedTables.delete(tableName);
			expandedTables = new Set(expandedTables);
		} else {
			expandedTables.add(tableName);
			expandedTables = new Set(expandedTables);
			
			// Load columns if not already loaded
			if (!tableColumns[tableName] && !loadingColumns.has(tableName)) {
				await loadTableColumns(tableName);
			}
		}
	}
	
	async function loadTableColumns(tableName: string) {
		if (!activeConnection) return;
		
		loadingColumns.add(tableName);
		loadingColumns = new Set(loadingColumns);
		
		try {
			const columns = await invoke<SchemaColumn[]>('get_table_columns', {
				connectionId: activeConnection.id,
				tableName
			});
			tableColumns[tableName] = columns;
		} catch (err) {
			console.error(`Failed to load columns for ${tableName}:`, err);
		} finally {
			loadingColumns.delete(tableName);
			loadingColumns = new Set(loadingColumns);
		}
	}
	
	function handleTableClick(tableName: string, schemaName?: string) {
		if (onTableSelect) {
			// If schema is provided and not 'public', use schema-qualified name
			if (schemaName && schemaName !== 'public') {
				onTableSelect(`${schemaName}.${tableName}`);
			} else {
				onTableSelect(tableName);
			}
		}
	}

	function toggleSchema(schemaName: string) {
		if (expandedSchemas.has(schemaName)) {
			expandedSchemas.delete(schemaName);
		} else {
			expandedSchemas.add(schemaName);
		}
		expandedSchemas = new Set(expandedSchemas);
	}
	
	function toggleSection(section: string) {
		if (expandedSections.has(section)) {
			expandedSections.delete(section);
		} else {
			expandedSections.add(section);
		}
		expandedSections = new Set(expandedSections);
	}
	
	function getColumnIcon(dataType: string) {
		const type = dataType.toLowerCase();
		if (type.includes('int') || type.includes('serial') || type.includes('numeric') || type.includes('decimal')) {
			return Hash;
		}
		if (type.includes('char') || type.includes('text') || type.includes('varchar')) {
			return Type;
		}
		return Columns;
	}

	// Group tables by schema
	function groupBySchema(tables: SchemaTable[]): Map<string, SchemaTable[]> {
		const grouped = new Map<string, SchemaTable[]>();
		for (const table of tables) {
			const schemaName = table.table_schema;
			if (!grouped.has(schemaName)) {
				grouped.set(schemaName, []);
			}
			grouped.get(schemaName)!.push(table);
		}
		return grouped;
	}

	async function copyTableSchema(tableName: string, event: MouseEvent) {
		event.stopPropagation();
		if (!activeConnection) return;

		try {
			const createStatement = await invoke<string>('get_table_create_statement', {
				connectionId: activeConnection.id,
				tableName,
				schemaName: null // Using default public schema
			});

			// Use Tauri's clipboard API
			await invoke('plugin:clipboard-manager|write_text', {
				text: createStatement
			});

			// Show feedback
			copiedTables.add(tableName);
			copiedTables = new Set(copiedTables);

			// Reset after 2 seconds
			setTimeout(() => {
				copiedTables.delete(tableName);
				copiedTables = new Set(copiedTables);
			}, 2000);
		} catch (err) {
			console.error(`Failed to copy schema for ${tableName}:`, err);
		}
	}
</script>

<div class="schema-panel h-full flex flex-col bg-surface-50-950 border-r border-surface-300-600">
	<div class="p-3 border-b border-surface-300-600 bg-surface-100-800">
		<div class="flex items-center gap-2 text-sm font-medium text-surface-200">
			<Database class="h-4 w-4" />
			<span>Schema</span>
		</div>
	</div>
	
	<div class="flex-1 overflow-y-auto p-2">
		{#if loading}
			<div class="flex items-center justify-center py-8 text-surface-400">
				<div class="flex items-center gap-2">
					<div class="animate-spin w-4 h-4 border-2 border-primary-500 border-t-transparent rounded-full"></div>
					<span class="text-sm">Loading schema...</span>
				</div>
			</div>
		{:else if error}
			<div class="p-4 text-red-400 text-sm">
				<p class="font-medium">Failed to load schema</p>
				<p class="mt-1 text-xs opacity-75">{error}</p>
				<button 
					onclick={loadSchema}
					class="mt-2 btn btn-sm btn-ghost-surface text-xs px-2 py-1"
				>
					Retry
				</button>
			</div>
		{:else if schema}
			<!-- Tables Section (Grouped by Schema) -->
			{#if schema.tables.length > 0}
				{@const tablesBySchema = groupBySchema(schema.tables)}
				<div class="mb-4">
					<button
						onclick={() => toggleSection('tables')}
						class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-surface-400 uppercase tracking-wide hover:bg-surface-200-700 rounded transition-colors"
					>
						{#if expandedSections.has('tables')}
							<ChevronDown class="h-3 w-3" />
						{:else}
							<ChevronRight class="h-3 w-3" />
						{/if}
						<Table class="h-3 w-3" />
						<span>Tables ({schema.tables.length})</span>
					</button>
					{#if expandedSections.has('tables')}
						{#each Array.from(tablesBySchema.entries()) as [schemaName, tables]}
							<!-- Schema group -->
							<div class="ml-1 mt-1">
								<button
									onclick={() => toggleSchema(schemaName)}
									class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-surface-300 hover:bg-surface-200-700 rounded transition-colors"
								>
									{#if expandedSchemas.has(schemaName)}
										<ChevronDown class="h-3 w-3" />
									{:else}
										<ChevronRight class="h-3 w-3" />
									{/if}
									<Folder class="h-3 w-3 text-amber-400" />
									<span>{schemaName}</span>
									<span class="ml-auto text-surface-500">({tables.length})</span>
								</button>

								{#if expandedSchemas.has(schemaName)}
									{#each tables as table (`${table.table_schema}.${table.table_name}`)}
										<div class="ml-3">
											<div class="w-full flex items-center gap-2 px-2 py-1.5 text-sm text-surface-300 hover:bg-surface-200-700 rounded transition-colors group">
												<button
													onclick={() => toggleTable(table.table_name)}
													class="flex items-center gap-2 flex-1"
												>
													{#if expandedTables.has(table.table_name)}
														<ChevronDown class="h-3 w-3 text-surface-500" />
													{:else}
														<ChevronRight class="h-3 w-3 text-surface-500" />
													{/if}
													<Table class="h-3 w-3 text-blue-400" />
													<span class="flex-1 text-left truncate">{table.table_name}</span>
												</button>
												<span class="text-xs text-surface-500 opacity-0 group-hover:opacity-100 transition-opacity">
													{table.column_count}
												</span>
												<button
													onclick={(event) => copyTableSchema(table.table_name, event)}
													class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-all duration-200 {copiedTables.has(table.table_name) ? '!opacity-100 !bg-green-500/20' : ''}"
													title="{copiedTables.has(table.table_name) ? 'Copied!' : 'Copy CREATE TABLE statement'}"
												>
													{#if copiedTables.has(table.table_name)}
														<Check class="h-3 w-3 text-green-400 animate-scale-in" />
													{:else}
														<Copy class="h-3 w-3" />
													{/if}
												</button>
												<button
													onclick={(e) => { e.stopPropagation(); handleTableClick(table.table_name, table.table_schema); }}
													class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity"
													title="Insert table name"
												>
													<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
														<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
													</svg>
												</button>
											</div>

											<!-- Columns -->
											{#if expandedTables.has(table.table_name)}
												<div class="ml-6 mt-1 space-y-0.5">
													{#if loadingColumns.has(table.table_name)}
														<div class="flex items-center gap-2 px-2 py-1 text-xs text-surface-500">
															<div class="animate-spin w-3 h-3 border border-primary-500 border-t-transparent rounded-full"></div>
															<span>Loading columns...</span>
														</div>
													{:else if tableColumns[table.table_name]}
														{#each tableColumns[table.table_name] as column (column.column_name)}
															<div class="flex items-center gap-2 px-2 py-1 text-xs text-surface-400 hover:bg-surface-200-700 rounded group">
																<svelte:component this={getColumnIcon(column.data_type)} class="h-3 w-3 text-surface-500" />
																<span class="font-mono text-surface-300">{column.column_name}</span>
																<span class="text-surface-500">{column.data_type}</span>
																{#if column.is_primary_key}
																	<Key class="h-2.5 w-2.5 text-yellow-500" title="Primary key" />
																{:else if column.is_nullable === 'NO'}
																	<Shield class="h-2.5 w-2.5 text-amber-500" title="Not null" />
																{/if}
																<button
																	onclick={() => handleTableClick(`${table.table_name}.${column.column_name}`, table.table_schema)}
																	class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity ml-auto"
																	title="Insert column name"
																>
																	<svg class="h-2.5 w-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
																		<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
																	</svg>
																</button>
															</div>
														{/each}
													{/if}
												</div>
											{/if}
										</div>
									{/each}
								{/if}
							</div>
						{/each}
					{/if}
				</div>
			{/if}
			
			<!-- Views Section -->
			{#if schema.views.length > 0}
				<div class="mb-4">
					<button
						onclick={() => toggleSection('views')}
						class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-surface-400 uppercase tracking-wide hover:bg-surface-200-700 rounded transition-colors"
					>
						{#if expandedSections.has('views')}
							<ChevronDown class="h-3 w-3" />
						{:else}
							<ChevronRight class="h-3 w-3" />
						{/if}
						<Eye class="h-3 w-3" />
						<span>Views ({schema.views.length})</span>
					</button>
					{#if expandedSections.has('views')}
						{#each schema.views as view (view.table_name)}
						<div class="ml-1">
							<div class="w-full flex items-center gap-2 px-2 py-1.5 text-sm text-surface-300 hover:bg-surface-200-700 rounded transition-colors group">
								<button
									onclick={() => toggleTable(view.table_name)}
									class="flex items-center gap-2 flex-1"
								>
									{#if expandedTables.has(view.table_name)}
										<ChevronDown class="h-3 w-3 text-surface-500" />
									{:else}
										<ChevronRight class="h-3 w-3 text-surface-500" />
									{/if}
									<Eye class="h-3 w-3 text-green-400" />
									<span class="flex-1 text-left truncate">{view.table_name}</span>
								</button>
								<span class="text-xs text-surface-500 opacity-0 group-hover:opacity-100 transition-opacity">
									{view.column_count}
								</span>
								<button
									onclick={(event) => copyTableSchema(view.table_name, event)}
									class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-all duration-200 {copiedTables.has(view.table_name) ? '!opacity-100 !bg-green-500/20' : ''}"
									title="{copiedTables.has(view.table_name) ? 'Copied!' : 'Copy CREATE VIEW statement'}"
								>
									{#if copiedTables.has(view.table_name)}
										<Check class="h-3 w-3 text-green-400 animate-scale-in" />
									{:else}
										<Copy class="h-3 w-3" />
									{/if}
								</button>
								<button
									onclick={(e) => { e.stopPropagation(); handleTableClick(view.table_name, view.table_schema); }}
									class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity"
									title="Insert view name"
								>
									<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
									</svg>
								</button>
							</div>
							
							<!-- View Columns -->
							{#if expandedTables.has(view.table_name)}
								<div class="ml-6 mt-1 space-y-0.5">
									{#if loadingColumns.has(view.table_name)}
										<div class="flex items-center gap-2 px-2 py-1 text-xs text-surface-500">
											<div class="animate-spin w-3 h-3 border border-primary-500 border-t-transparent rounded-full"></div>
											<span>Loading columns...</span>
										</div>
									{:else if tableColumns[view.table_name]}
										{#each tableColumns[view.table_name] as column (column.column_name)}
											<div class="flex items-center gap-2 px-2 py-1 text-xs text-surface-400 hover:bg-surface-200-700 rounded group">
												<svelte:component this={getColumnIcon(column.data_type)} class="h-3 w-3 text-surface-500" />
												<span class="font-mono text-surface-300">{column.column_name}</span>
												<span class="text-surface-500">{column.data_type}</span>
												{#if column.is_primary_key}
													<Key class="h-2.5 w-2.5 text-yellow-500" title="Primary key" />
												{:else if column.is_nullable === 'NO'}
													<Shield class="h-2.5 w-2.5 text-amber-500" title="Not null" />
												{/if}
												<button
													onclick={() => handleTableClick(`${view.table_name}.${column.column_name}`, view.table_schema)}
													class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity ml-auto"
													title="Insert column name"
												>
													<svg class="h-2.5 w-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
														<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
													</svg>
												</button>
											</div>
										{/each}
									{/if}
								</div>
							{/if}
						</div>
					{/each}
					{/if}
				</div>
			{/if}

			<!-- Materialized Views Section -->
			{#if schema.materialized_views.length > 0}
				<div class="mb-4">
					<button onclick={() => toggleSection('materializedViews')} class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-surface-400 uppercase tracking-wide hover:text-surface-300 hover:bg-surface-200-700 rounded transition-colors">
						{#if expandedSections.has('materializedViews')}
							<ChevronDown class="h-3 w-3" />
						{:else}
							<ChevronRight class="h-3 w-3" />
						{/if}
						<Layers class="h-3 w-3" />
						<span>Materialized Views ({schema.materialized_views.length})</span>
					</button>
					{#if expandedSections.has('materializedViews')}
					{#each schema.materialized_views as view (view.table_name)}
						<div class="ml-1">
							<div class="flex items-center gap-2 px-2 py-1.5 text-sm text-surface-300 hover:bg-surface-200-700 rounded transition-colors group">
								<Layers class="h-3 w-3 text-purple-400" />
								<span class="flex-1 text-left truncate">{view.table_name}</span>
								<span class="text-xs text-surface-500 opacity-0 group-hover:opacity-100 transition-opacity">
									{view.column_count}
								</span>
								<button
									onclick={() => handleTableClick(view.table_name)}
									class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity"
									title="Insert materialized view name"
								>
									<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
									</svg>
								</button>
							</div>
						</div>
					{/each}
					{/if}
				</div>
			{/if}

			<!-- Indexes Section -->
			{#if schema.indexes.length > 0}
				<div class="mb-4">
					<button onclick={() => toggleSection('indexes')} class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-surface-400 uppercase tracking-wide hover:text-surface-300 hover:bg-surface-200-700 rounded transition-colors">
						{#if expandedSections.has('indexes')}
							<ChevronDown class="h-3 w-3" />
						{:else}
							<ChevronRight class="h-3 w-3" />
						{/if}
						<Zap class="h-3 w-3" />
						<span>Indexes ({schema.indexes.length})</span>
					</button>
					{#if expandedSections.has('indexes')}
					{#each schema.indexes as index (index.index_name)}
						<div class="ml-1">
							<div class="flex items-center gap-2 px-2 py-1.5 text-sm text-surface-300 hover:bg-surface-200-700 rounded transition-colors group">
								<Zap class="h-3 w-3 {index.is_primary ? 'text-yellow-400' : index.is_unique ? 'text-orange-400' : 'text-cyan-400'}" />
								<span class="flex-1 text-left truncate font-mono text-xs">{index.index_name}</span>
								<span class="text-xs text-surface-500 opacity-0 group-hover:opacity-100 transition-opacity">
									{index.table_name}
								</span>
								<button
									onclick={() => handleTableClick(index.index_name)}
									class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity"
									title="Insert index name"
								>
									<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
									</svg>
								</button>
							</div>
						</div>
					{/each}
					{/if}
				</div>
			{/if}

			<!-- Functions Section -->
			{#if schema.functions.length > 0}
				<div class="mb-4">
					<button onclick={() => toggleSection('functions')} class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-surface-400 uppercase tracking-wide hover:text-surface-300 hover:bg-surface-200-700 rounded transition-colors">
						{#if expandedSections.has('functions')}
							<ChevronDown class="h-3 w-3" />
						{:else}
							<ChevronRight class="h-3 w-3" />
						{/if}
						<Settings class="h-3 w-3" />
						<span>Functions ({schema.functions.length})</span>
					</button>
					{#if expandedSections.has('functions')}
					{#each schema.functions as func (func.function_name)}
						<div class="ml-1">
							<div class="flex items-center gap-2 px-2 py-1.5 text-sm text-surface-300 hover:bg-surface-200-700 rounded transition-colors group">
								<Settings class="h-3 w-3 text-indigo-400" />
								<span class="flex-1 text-left truncate font-mono text-xs">{func.function_name}</span>
								<span class="text-xs text-surface-500 opacity-0 group-hover:opacity-100 transition-opacity">
									{func.function_type}
								</span>
								<button
									onclick={() => handleTableClick(`${func.function_name}()`)}
									class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity"
									title="Insert function call"
								>
									<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
									</svg>
								</button>
							</div>
						</div>
					{/each}
					{/if}
				</div>
			{/if}

			<!-- Sequences Section -->
			{#if schema.sequences.length > 0}
				<div class="mb-4">
					<button onclick={() => toggleSection('sequences')} class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-surface-400 uppercase tracking-wide hover:text-surface-300 hover:bg-surface-200-700 rounded transition-colors">
						{#if expandedSections.has('sequences')}
							<ChevronDown class="h-3 w-3" />
						{:else}
							<ChevronRight class="h-3 w-3" />
						{/if}
						<Hash class="h-3 w-3" />
						<span>Sequences ({schema.sequences.length})</span>
					</button>
					{#if expandedSections.has('sequences')}
					{#each schema.sequences as seq (seq.sequence_name)}
						<div class="ml-1">
							<div class="flex items-center gap-2 px-2 py-1.5 text-sm text-surface-300 hover:bg-surface-200-700 rounded transition-colors group">
								<Hash class="h-3 w-3 text-pink-400" />
								<span class="flex-1 text-left truncate font-mono text-xs">{seq.sequence_name}</span>
								<span class="text-xs text-surface-500 opacity-0 group-hover:opacity-100 transition-opacity">
									{seq.data_type}
								</span>
								<button
									onclick={() => handleTableClick(seq.sequence_name)}
									class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity"
									title="Insert sequence name"
								>
									<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
									</svg>
								</button>
							</div>
						</div>
					{/each}
					{/if}
				</div>
			{/if}

			<!-- Foreign Keys Section -->
			{#if schema.foreign_keys.length > 0}
				<div class="mb-4">
					<button onclick={() => toggleSection('foreign_keys')} class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-surface-400 uppercase tracking-wide hover:text-surface-300 hover:bg-surface-200-700 rounded transition-colors">
						{#if expandedSections.has('foreign_keys')}
							<ChevronDown class="h-3 w-3" />
						{:else}
							<ChevronRight class="h-3 w-3" />
						{/if}
						<Link class="h-3 w-3" />
						<span>Foreign Keys ({schema.foreign_keys.length})</span>
					</button>
					{#if expandedSections.has('foreign_keys')}
					{#each schema.foreign_keys as fk (fk.constraint_name)}
						<div class="ml-1">
							<div class="flex items-center gap-2 px-2 py-1.5 text-sm text-surface-300 hover:bg-surface-200-700 rounded transition-colors group">
								<Link class="h-3 w-3 text-emerald-400" />
								<span class="flex-1 text-left truncate font-mono text-xs">
									{fk.table_name}.{fk.column_name} → {fk.foreign_table_name}.{fk.foreign_column_name}
								</span>
								<button
									onclick={() => handleTableClick(`${fk.table_name}.${fk.column_name}`)}
									class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity"
									title="Insert column name"
								>
									<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
									</svg>
								</button>
							</div>
						</div>
					{/each}
					{/if}
				</div>
			{/if}

			<!-- Enums Section -->
			{#if schema.enums.length > 0}
				<div class="mb-4">
					<button onclick={() => toggleSection('enums')} class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-surface-400 uppercase tracking-wide hover:text-surface-300 hover:bg-surface-200-700 rounded transition-colors">
						{#if expandedSections.has('enums')}
							<ChevronDown class="h-3 w-3" />
						{:else}
							<ChevronRight class="h-3 w-3" />
						{/if}
						<Package class="h-3 w-3" />
						<span>Enums ({schema.enums.length})</span>
					</button>
					{#if expandedSections.has('enums')}
					{#each schema.enums as enumType (enumType.type_name)}
						<div class="ml-1">
							<div class="flex items-center gap-2 px-2 py-1.5 text-sm text-surface-300 hover:bg-surface-200-700 rounded transition-colors group">
								<Package class="h-3 w-3 text-violet-400" />
								<span class="flex-1 text-left truncate font-mono text-xs">{enumType.type_name}</span>
								<span class="text-xs text-surface-500 opacity-0 group-hover:opacity-100 transition-opacity">
									{enumType.enum_values.length} values
								</span>
								<button
									onclick={() => handleTableClick(enumType.type_name)}
									class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity"
									title="Insert enum type"
								>
									<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
									</svg>
								</button>
							</div>
						</div>
					{/each}
					{/if}
				</div>
			{/if}

			<!-- Schemas Section -->
			{#if schema.schemas.length > 1}
				<div class="mb-4">
					<button onclick={() => toggleSection('schemas')} class="w-full flex items-center gap-2 px-2 py-1 text-xs font-medium text-surface-400 uppercase tracking-wide hover:text-surface-300 hover:bg-surface-200-700 rounded transition-colors">
						{#if expandedSections.has('schemas')}
							<ChevronDown class="h-3 w-3" />
						{:else}
							<ChevronRight class="h-3 w-3" />
						{/if}
						<Folder class="h-3 w-3" />
						<span>Schemas ({schema.schemas.length})</span>
					</button>
					{#if expandedSections.has('schemas')}
					{#each schema.schemas as schemaItem (schemaItem.schema_name)}
						<div class="ml-1">
							<div class="flex items-center gap-2 px-2 py-1.5 text-sm text-surface-300 hover:bg-surface-200-700 rounded transition-colors group">
								<Folder class="h-3 w-3 text-amber-400" />
								<span class="flex-1 text-left truncate font-mono text-xs">{schemaItem.schema_name}</span>
								<span class="text-xs text-surface-500 opacity-0 group-hover:opacity-100 transition-opacity">
									{schemaItem.owner}
								</span>
								<button
									onclick={() => handleTableClick(schemaItem.schema_name)}
									class="opacity-0 group-hover:opacity-100 hover:bg-surface-300-600 rounded p-0.5 transition-opacity"
									title="Insert schema name"
								>
									<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
									</svg>
								</button>
							</div>
						</div>
					{/each}
					{/if}
				</div>
			{/if}
			
			{#if schema.tables.length === 0 && schema.views.length === 0 && schema.materialized_views.length === 0 && schema.functions.length === 0}
				<div class="flex items-center justify-center py-8 text-surface-500">
					<div class="text-center">
						<Database class="h-8 w-8 mx-auto mb-2 opacity-50" />
						<p class="text-sm">No database objects found</p>
					</div>
				</div>
			{/if}
		{:else if !activeConnection}
			<div class="flex items-center justify-center py-8 text-surface-500">
				<div class="text-center">
					<Database class="h-8 w-8 mx-auto mb-2 opacity-50" />
					<p class="text-sm">Connect to a database to view schema</p>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.schema-panel {
		min-width: 250px;
		max-width: 400px;
		width: 280px;
	}

	@keyframes scale-in {
		from {
			transform: scale(0);
		}
		to {
			transform: scale(1);
		}
	}

	:global(.animate-scale-in) {
		animation: scale-in 0.2s ease-out;
	}
</style>