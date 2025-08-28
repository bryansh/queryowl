<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import loader from '@monaco-editor/loader';
	import { Play, Loader2, Save, FileDown } from 'lucide-svelte';
	
	let { 
		value = $bindable('-- Write your SQL query here\nSELECT * FROM '),
		theme = 'vs-dark',
		height = '400px',
		onExecute,
		isExecuting = false
	}: {
		value?: string;
		theme?: string;
		height?: string;
		onExecute?: ((sql: string) => Promise<void>) | undefined;
		isExecuting?: boolean;
	} = $props();
	
	let container: HTMLDivElement;
	let editor: any;
	let monaco: any;
	
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
	
	function formatSql() {
		// Basic SQL formatting - you could enhance this with a proper SQL formatter
		const sql = editor.getValue();
		const formatted = sql
			.replace(/\s+/g, ' ')
			.replace(/\s*,\s*/g, ',\n  ')
			.replace(/\sSELECT\s/gi, 'SELECT\n  ')
			.replace(/\sFROM\s/gi, '\nFROM ')
			.replace(/\sWHERE\s/gi, '\nWHERE ')
			.replace(/\sJOIN\s/gi, '\nJOIN ')
			.replace(/\sLEFT JOIN\s/gi, '\nLEFT JOIN ')
			.replace(/\sRIGHT JOIN\s/gi, '\nRIGHT JOIN ')
			.replace(/\sINNER JOIN\s/gi, '\nINNER JOIN ')
			.replace(/\sGROUP BY\s/gi, '\nGROUP BY ')
			.replace(/\sORDER BY\s/gi, '\nORDER BY ')
			.replace(/\sHAVING\s/gi, '\nHAVING ')
			.replace(/\sLIMIT\s/gi, '\nLIMIT ')
			.replace(/\sOFFSET\s/gi, '\nOFFSET ');
		
		editor.setValue(formatted);
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
</script>

<div class="sql-editor-container h-full flex flex-col bg-slate-900">
	<div class="editor-toolbar flex items-center justify-between px-4 py-3 border-b border-slate-700 bg-slate-800">
		<div class="flex items-center gap-3">
			<button 
				onclick={handleExecute}
				disabled={!onExecute || isExecuting}
				title="Execute Query (Cmd+Enter)"
				class="btn btn-primary btn-sm px-4 py-2 font-medium"
			>
				{#if isExecuting}
					<Loader2 class="h-4 w-4 mr-2 animate-spin" />
					Executing...
				{:else}
					<Play class="h-4 w-4 mr-2" />
					Run
				{/if}
			</button>
			<button 
				onclick={formatSql}
				title="Format SQL"
				class="btn btn-ghost btn-sm px-3 py-2"
			>
				Format
			</button>
		</div>
		<div class="text-xs text-slate-400 font-medium">
			Query #1
		</div>
	</div>
	<div 
		bind:this={container} 
		class="flex-1 editor-container"
	></div>
</div>

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