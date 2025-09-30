<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import loader from '@monaco-editor/loader';
	import { Play, Loader2, Save, FileDown, Bookmark } from 'lucide-svelte';
	
	let { 
		value = $bindable('-- Write your SQL query here\nSELECT * FROM '),
		theme = 'vs-dark',
		height = '400px',
		onExecute,
		isExecuting = false,
		onReady,
		onSave,
		schema
	}: {
		value?: string;
		theme?: string;
		height?: string;
		onExecute?: ((sql: string) => Promise<void>) | undefined;
		isExecuting?: boolean;
		onReady?: (() => void) | undefined;
		onSave?: ((sql: string) => void) | undefined;
		schema?: any;
	} = $props();
	
	let container: HTMLDivElement;
	let editor: any;
	let monaco: any;
	let showSaveDialog = $state(false);
	let queryName = $state('');
	let queryDescription = $state('');
	let completionProvider: any = null;
	
	onMount(async () => {
		// Load Monaco Editor
		monaco = await loader.init();
		
		// Configure SQL language features with dynamic schema completion
		const registerCompletionProvider = () => {
			// Dispose previous provider if exists
			if (completionProvider) {
				completionProvider.dispose();
			}
			
			completionProvider = monaco.languages.registerCompletionItemProvider('sql', {
				provideCompletionItems: (model: any, position: any) => {
					// Get the text before the cursor to understand context
					const textUntilPosition = model.getValueInRange({
						startLineNumber: 1,
						startColumn: 1,
						endLineNumber: position.lineNumber,
						endColumn: position.column,
					});
					
					// Check if we're typing after a table name (for column suggestions)
					const words = textUntilPosition.split(/\s+/);
					const lastWord = words[words.length - 1];
					const prevWord = words[words.length - 2];
					
					// Check for table.column pattern
					const columnMatch = lastWord?.match(/^(\w+)\.$/); 
					const tableName = columnMatch ? columnMatch[1] : null;
					
					let suggestions = [
						// SQL Keywords
						...['SELECT', 'FROM', 'WHERE', 'JOIN', 'LEFT JOIN', 'RIGHT JOIN', 'INNER JOIN', 
						    'ON', 'GROUP BY', 'ORDER BY', 'HAVING', 'LIMIT', 'OFFSET', 'DISTINCT',
						    'INSERT INTO', 'UPDATE', 'DELETE FROM', 'CREATE TABLE', 'DROP TABLE',
						    'ALTER TABLE', 'CREATE INDEX', 'DROP INDEX', 'UNION', 'EXCEPT', 'INTERSECT',
						    'AS', 'AND', 'OR', 'NOT', 'IN', 'EXISTS', 'BETWEEN', 'LIKE', 'IS NULL',
						    'IS NOT NULL', 'COUNT', 'SUM', 'AVG', 'MIN', 'MAX', 'CASE', 'WHEN', 'THEN',
						    'ELSE', 'END', 'CAST', 'COALESCE', 'NULLIF'].map(keyword => ({
							label: keyword,
							kind: monaco.languages.CompletionItemKind.Keyword,
							insertText: keyword,
							detail: 'SQL Keyword'
						})),
						// PostgreSQL specific functions
						...['NOW()', 'CURRENT_DATE', 'CURRENT_TIME', 'CURRENT_TIMESTAMP',
						    'date_trunc()', 'extract()', 'to_char()', 'to_date()', 'to_timestamp()',
						    'array_agg()', 'string_agg()', 'json_build_object()', 'jsonb_build_object()',
						    'row_number()', 'rank()', 'dense_rank()', 'lag()', 'lead()'].map(func => ({
							label: func,
							kind: monaco.languages.CompletionItemKind.Function,
							insertText: func,
							detail: 'PostgreSQL Function'
						}))
					];
					
					// If we're typing columns after a table name (e.g., "users."), show columns for that table
					if (tableName && schema) {
						suggestions = []; // Clear other suggestions to focus on columns
						
						// Find table and show its columns
						const allTables = [...(schema.tables || []), ...(schema.views || []), ...(schema.materialized_views || [])];
						const table = allTables.find((t: any) => t.table_name.toLowerCase() === tableName.toLowerCase());
						
						if (table) {
							// We need to fetch columns for this specific table
							// For now, we'll return a generic suggestion that columns are available
							suggestions.push({
								label: '*',
								kind: monaco.languages.CompletionItemKind.Field,
								insertText: '*',
								detail: 'All columns',
								documentation: `All columns from ${table.table_name} (${table.column_count} columns available)`
							});
							
							// Add placeholder for common column names
							const commonColumns = ['id', 'name', 'email', 'created_at', 'updated_at', 'deleted_at', 'status', 'type', 'description'];
							suggestions.push(...commonColumns.map(col => ({
								label: col,
								kind: monaco.languages.CompletionItemKind.Field,
								insertText: col,
								detail: `Possible column from ${table.table_name}`,
								documentation: 'Common column name - verify existence'
							})));
						}
						return { suggestions };
					}
					
					// Add database schema suggestions if available
					if (schema) {
						// Add tables (both qualified and unqualified names)
						if (schema.tables) {
							schema.tables.forEach((table: any) => {
								const schemaLabel = table.table_schema ? ` [${table.table_schema}]` : '';

								// Always add the unqualified table name
								suggestions.push({
									label: table.table_name,
									kind: monaco.languages.CompletionItemKind.Class,
									insertText: table.table_name,
									detail: `Table${schemaLabel} (${table.column_count} columns)`,
									documentation: `Base table with ${table.column_count} columns${table.table_schema ? ` in schema ${table.table_schema}` : ''}`
								});

								// For non-public schemas, also add the fully-qualified name
								if (table.table_schema && table.table_schema !== 'public') {
									suggestions.push({
										label: `${table.table_schema}.${table.table_name}`,
										kind: monaco.languages.CompletionItemKind.Class,
										insertText: `${table.table_schema}.${table.table_name}`,
										detail: `Table [${table.table_schema}] (${table.column_count} columns)`,
										documentation: `Fully-qualified: ${table.table_schema}.${table.table_name} - Base table with ${table.column_count} columns`,
										sortText: `1_${table.table_schema}.${table.table_name}` // Sort qualified names first
									});
								}
							});
						}

						// Add views (both qualified and unqualified names)
						if (schema.views) {
							schema.views.forEach((view: any) => {
								const schemaLabel = view.table_schema ? ` [${view.table_schema}]` : '';

								// Always add the unqualified view name
								suggestions.push({
									label: view.table_name,
									kind: monaco.languages.CompletionItemKind.Interface,
									insertText: view.table_name,
									detail: `View${schemaLabel} (${view.column_count} columns)`,
									documentation: `Database view with ${view.column_count} columns${view.table_schema ? ` in schema ${view.table_schema}` : ''}`
								});

								// For non-public schemas, also add the fully-qualified name
								if (view.table_schema && view.table_schema !== 'public') {
									suggestions.push({
										label: `${view.table_schema}.${view.table_name}`,
										kind: monaco.languages.CompletionItemKind.Interface,
										insertText: `${view.table_schema}.${view.table_name}`,
										detail: `View [${view.table_schema}] (${view.column_count} columns)`,
										documentation: `Fully-qualified: ${view.table_schema}.${view.table_name} - Database view with ${view.column_count} columns`,
										sortText: `1_${view.table_schema}.${view.table_name}` // Sort qualified names first
									});
								}
							});
						}

						// Add materialized views (both qualified and unqualified names)
						if (schema.materialized_views) {
							schema.materialized_views.forEach((mv: any) => {
								const schemaLabel = mv.table_schema ? ` [${mv.table_schema}]` : '';

								// Always add the unqualified name
								suggestions.push({
									label: mv.table_name,
									kind: monaco.languages.CompletionItemKind.Struct,
									insertText: mv.table_name,
									detail: `Materialized View${schemaLabel} (${mv.column_count} columns)`,
									documentation: `Materialized view with ${mv.column_count} columns${mv.table_schema ? ` in schema ${mv.table_schema}` : ''}`
								});

								// For non-public schemas, also add the fully-qualified name
								if (mv.table_schema && mv.table_schema !== 'public') {
									suggestions.push({
										label: `${mv.table_schema}.${mv.table_name}`,
										kind: monaco.languages.CompletionItemKind.Struct,
										insertText: `${mv.table_schema}.${mv.table_name}`,
										detail: `Materialized View [${mv.table_schema}] (${mv.column_count} columns)`,
										documentation: `Fully-qualified: ${mv.table_schema}.${mv.table_name} - Materialized view with ${mv.column_count} columns`,
										sortText: `1_${mv.table_schema}.${mv.table_name}` // Sort qualified names first
									});
								}
							});
						}
						
						// Add functions
						if (schema.functions) {
							suggestions.push(...schema.functions.map((func: any) => ({
								label: `${func.function_name}()`,
								kind: monaco.languages.CompletionItemKind.Function,
								insertText: `${func.function_name}()`,
								detail: `${func.function_type} → ${func.return_type}`,
								documentation: `Database ${func.function_type.toLowerCase()} returning ${func.return_type}`
							})));
						}
						
						// Add sequences
						if (schema.sequences) {
							suggestions.push(...schema.sequences.map((seq: any) => ({
								label: seq.sequence_name,
								kind: monaco.languages.CompletionItemKind.Constant,
								insertText: seq.sequence_name,
								detail: `Sequence (${seq.data_type})`,
								documentation: `Database sequence of type ${seq.data_type}`
							})));
						}
						
						// Add enums
						if (schema.enums) {
							suggestions.push(...schema.enums.map((enumType: any) => ({
								label: enumType.type_name,
								kind: monaco.languages.CompletionItemKind.Enum,
								insertText: enumType.type_name,
								detail: `Enum (${enumType.enum_values.length} values)`,
								documentation: `Enum type: ${enumType.enum_values.join(', ')}`
							})));
						}

						// Add schemas
						if (schema.schemas) {
							suggestions.push(...schema.schemas.map((schemaItem: any) => ({
								label: schemaItem.schema_name,
								kind: monaco.languages.CompletionItemKind.Module,
								insertText: schemaItem.schema_name,
								detail: `Schema (owner: ${schemaItem.owner})`,
								documentation: `Database schema owned by ${schemaItem.owner}`
							})));
						}
					}
					
					return { suggestions };
				}
			});
		};
		
		// Initial registration
		registerCompletionProvider();
		
		// Create editor instance
		editor = monaco.editor.create(container, {
			value: value,
			language: 'sql',
			theme: 'vs-dark',
			minimap: { enabled: false },
			automaticLayout: true,
			fontSize: 14,
			lineNumbers: 'on',
			roundedSelection: false,
			scrollBeyondLastLine: false,
			readOnly: false,
			cursorStyle: 'line',
			wordWrap: 'on',
			suggest: {
				insertMode: 'replace',
				showKeywords: true
			},
			quickSuggestions: {
				other: true,
				comments: false,
				strings: false
			},
			parameterHints: {
				enabled: true
			},
			suggestOnTriggerCharacters: true,
			acceptSuggestionOnEnter: 'on',
			tabCompletion: 'on',
			wordBasedSuggestions: 'allDocuments'
		});
		
		// Update value when editor content changes
		editor.onDidChangeModelContent(() => {
			value = editor.getValue();
		});
		
		// Add keyboard shortcuts
		editor.addAction({
			id: 'execute-query',
			label: 'Execute Query',
			keybindings: [
				monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter
			],
			run: () => {
				if (onExecute && !isExecuting) {
					handleExecute();
				}
			}
		});
		
		// Notify parent component that the editor is ready
		if (onReady) {
			onReady();
		}
	});
	
	onDestroy(() => {
		if (editor) {
			editor.dispose();
		}
	});
	
	async function handleExecute() {
		if (onExecute && !isExecuting) {
			const sql = editor.getValue();
			if (sql.trim()) {
				await onExecute(sql);
			}
		}
	}
	
	export function getSelectedText() {
		const selection = editor.getSelection();
		return editor.getModel().getValueInRange(selection);
	}
	
	export function getValue() {
		return editor?.getValue() || value;
	}
	
	export function setValue(newValue: string) {
		if (editor) {
			editor.setValue(newValue);
		}
		value = newValue;
	}
	
	function handleSave() {
		if (onSave) {
			showSaveDialog = true;
			queryName = '';
			queryDescription = '';
		}
	}
	
	function saveQuery() {
		if (queryName && onSave) {
			onSave({
				name: queryName,
				sql: editor.getValue(),
				description: queryDescription || undefined
			});
			showSaveDialog = false;
		}
	}
	
	export function focus() {
		if (editor) {
			// Move cursor to end of content and focus
			const model = editor.getModel();
			if (model) {
				const lineCount = model.getLineCount();
				const lastLineLength = model.getLineMaxColumn(lineCount);
				const position = { lineNumber: lineCount, column: lastLineLength };
				editor.setPosition(position);
			}
			editor.focus();
		}
	}
</script>

<div class="sql-editor-container h-full relative bg-slate-900">
	<div 
		bind:this={container} 
		class="h-full editor-container"
	></div>
	
	<!-- Floating Action Buttons -->
	<div class="absolute bottom-4 right-4 flex items-center gap-2 z-10">
		{#if onSave}
			<button 
				onclick={handleSave}
				title="Save Query"
				class="btn btn-filled-surface bg-surface-700 hover:bg-surface-600 text-white rounded-full shadow-lg transition-all duration-200 hover:scale-105 group overflow-hidden"
			>
				<div class="flex items-center transition-all duration-200 group-hover:px-4 p-2">
					<Bookmark class="h-5 w-5 flex-shrink-0" />
					<span class="ml-2 whitespace-nowrap opacity-0 w-0 group-hover:opacity-100 group-hover:w-auto transition-all duration-200 font-medium text-sm">Save</span>
				</div>
			</button>
		{/if}
		
		<button 
			onclick={handleExecute}
			disabled={!onExecute || isExecuting}
			title="Execute Query (Cmd+Enter)"
			class="btn btn-filled-primary bg-primary-600 hover:bg-primary-700 text-white rounded-full shadow-lg transition-all duration-200 disabled:opacity-50 hover:scale-105 group overflow-hidden"
		>
			<div class="flex items-center transition-all duration-200 group-hover:px-4 p-2">
				{#if isExecuting}
					<Loader2 class="h-5 w-5 animate-spin flex-shrink-0" />
					<span class="ml-2 whitespace-nowrap opacity-0 w-0 group-hover:opacity-100 group-hover:w-auto transition-all duration-200 font-medium text-sm">Executing</span>
				{:else}
					<Play class="h-5 w-5 flex-shrink-0" />
					<span class="ml-2 whitespace-nowrap opacity-0 w-0 group-hover:opacity-100 group-hover:w-auto transition-all duration-200 font-medium text-sm">Execute</span>
				{/if}
			</div>
		</button>
	</div>
</div>

<!-- Save Query Dialog -->
{#if showSaveDialog}
	<div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
		<div class="card p-6 max-w-md mx-4 shadow-2xl">
			<h3 class="text-xl font-semibold mb-4">Save Query</h3>
			
			<div class="space-y-4">
				<div>
					<label for="query-name" class="block text-sm font-medium mb-2">Query Name *</label>
					<input
						id="query-name"
						type="text"
						bind:value={queryName}
						placeholder="e.g., Get Active Users"
						class="w-full px-3 py-2 bg-surface-50-950 border border-surface-300-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
						autofocus
					/>
				</div>
				
				<div>
					<label for="query-description" class="block text-sm font-medium mb-2">Description (Optional)</label>
					<textarea
						id="query-description"
						bind:value={queryDescription}
						placeholder="Describe what this query does..."
						rows="3"
						class="w-full px-3 py-2 bg-surface-50-950 border border-surface-300-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
					></textarea>
				</div>
			</div>
			
			<div class="flex gap-3 mt-6">
				<button
					onclick={saveQuery}
					disabled={!queryName}
					class="btn btn-filled-primary flex-1"
				>
					<Bookmark class="h-4 w-4 mr-2" />
					Save Query
				</button>
				<button
					onclick={() => showSaveDialog = false}
					class="btn btn-ghost-surface"
				>
					Cancel
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.sql-editor-container {
		width: 100%;
	}
	
	.editor-container {
		width: 100%;
		min-height: 200px;
	}
	
	kbd {
		font-family: monospace;
		border: 1px solid var(--border);
	}
</style>