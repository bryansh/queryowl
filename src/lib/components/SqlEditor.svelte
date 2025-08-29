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
		onSave
	}: {
		value?: string;
		theme?: string;
		height?: string;
		onExecute?: ((sql: string) => Promise<void>) | undefined;
		isExecuting?: boolean;
		onReady?: (() => void) | undefined;
		onSave?: ((sql: string) => void) | undefined;
	} = $props();
	
	let container: HTMLDivElement;
	let editor: any;
	let monaco: any;
	let showSaveDialog = $state(false);
	let queryName = $state('');
	let queryDescription = $state('');
	
	onMount(async () => {
		// Load Monaco Editor
		monaco = await loader.init();
		
		// Configure SQL language features
		monaco.languages.registerCompletionItemProvider('sql', {
			provideCompletionItems: (model: any, position: any) => {
				const suggestions = [
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
				
				return { suggestions };
			}
		});
		
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