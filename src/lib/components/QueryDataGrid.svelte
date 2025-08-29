<script lang="ts">
	import { Grid } from 'wx-svelte-grid';
	import { Download, Copy, CheckCircle, AlertCircle, Table } from 'lucide-svelte';
	
	let {
		data = [],
		error = null,
		executionTime = null,
		maxHeight = '500px'
	}: {
		data?: any[];
		error?: string | null;
		executionTime?: number | null;
		maxHeight?: string;
	} = $props();
	
	let copied = $state(false);
	let grid: any;
	
	// Transform data for SVAR Grid format with value formatting
	let gridData = $derived.by(() => {
		if (!data || data.length === 0) return [];
		return data.map((row, index) => {
			const formattedRow: any = { id: index + 1 };
			
			Object.entries(row).forEach(([key, value]) => {
				if (value === null || value === undefined) {
					formattedRow[key] = 'NULL';
				} else if (typeof value === 'boolean') {
					formattedRow[key] = value ? '✓' : '✗';
				} else if (typeof value === 'object') {
					formattedRow[key] = JSON.stringify(value, null, 2);
				} else if (typeof value === 'string' && !isNaN(Date.parse(value)) && value.match(/\d{4}-\d{2}-\d{2}/)) {
					formattedRow[key] = new Date(value).toLocaleString();
				} else {
					formattedRow[key] = value;
				}
			});
			
			return formattedRow;
		});
	});
	
	// Generate columns from data - simplified approach
	let columns = $derived.by(() => {
		if (!data || data.length === 0) return [];
		
		const firstRow = data[0];
		return Object.keys(firstRow).map(key => ({
			id: key,
			header: key,
			width: 150,
			flexgrow: 1,
			resize: true,
			sort: true
		}));
	});
	
	// Export functions
	function exportToCsv() {
		if (!data || data.length === 0) return;
		
		const headers = Object.keys(data[0]);
		const rows = data.map(row => 
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
	
	function copyToClipboard() {
		if (!data || data.length === 0) return;
		
		const headers = Object.keys(data[0]);
		const rows = data.map(row => 
			headers.map(header => {
				const value = row[header];
				if (value === null) return 'NULL';
				if (typeof value === 'object') return JSON.stringify(value);
				return String(value);
			}).join('\t')
		);
		
		const tsvContent = [headers.join('\t'), ...rows].join('\n');
		navigator.clipboard.writeText(tsvContent);
		copied = true;
		setTimeout(() => copied = false, 2000);
	}
	
	// Handle cell selection for copying
	function handleCellClick(ev: any) {
		if (ev.detail && ev.detail.cell) {
			const { row, column } = ev.detail.cell;
			if (data && data[row]) {
				const value = data[row][column.id];
				// Double click to copy cell value
				if (ev.detail.event?.detail === 2) {
					const textValue = value === null ? 'NULL' : 
									  typeof value === 'object' ? JSON.stringify(value, null, 2) : 
									  String(value);
					navigator.clipboard.writeText(textValue);
					copied = true;
					setTimeout(() => copied = false, 2000);
				}
			}
		}
	}
</script>

<div class="query-data-grid h-full flex flex-col bg-background">
	{#if error}
		<div class="p-6 bg-red-900/20 text-red-400 border-b border-red-800 rounded-t-lg">
			<div class="flex items-center gap-3">
				<AlertCircle class="h-6 w-6" />
				<span class="text-lg font-medium">Query Error</span>
			</div>
			<pre class="mt-4 text-base whitespace-pre-wrap font-mono bg-red-900/30 p-4 rounded-lg">{error}</pre>
		</div>
	{:else if data && data.length > 0}
		<!-- Results Grid - no toolbar, status info moved to main status bar -->
		<div class="flex-1 grid-container">
			<Grid 
				bind:this={grid}
				data={gridData} 
				{columns}
				resizable={true}
				sortable={true}
				filter={true}
				select={true}
				pager={true}
				onClick={handleCellClick}
			/>
		</div>
	{:else}
		<div class="flex-1 flex items-center justify-center text-slate-400">
			<div class="text-center">
				<Table class="h-16 w-16 mx-auto mb-4 opacity-50 text-primary-500" />
				<p class="text-xl font-medium">No results to display</p>
				<p class="text-base mt-2 opacity-75">Execute a SQL query to see results here</p>
			</div>
		</div>
	{/if}
</div>

<style>
	.query-data-grid {
		display: flex;
		flex-direction: column;
		height: 100%;
	}
	
	.grid-container {
		flex: 1;
		overflow: hidden;
		position: relative;
	}
	
	/* Beekeeper Studio inspired SVAR Grid styling */
	:global(.wx-grid) {
		--wx-grid-background: rgb(15 23 42);
		--wx-grid-text-color: rgb(203 213 225);
		--wx-grid-header-background: rgb(30 41 59);
		--wx-grid-header-text-color: rgb(148 163 184);
		--wx-grid-row-hover: rgb(51 65 85);
		--wx-grid-row-selected: rgb(59 130 246 / 0.3);
		--wx-grid-border-color: rgb(71 85 105);
		font-size: 0.8rem;
		font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
		background: rgb(15 23 42);
		border: 1px solid rgb(71 85 105);
	}
	
	:global(.wx-grid .wx-cell) {
		padding: 0.75rem 1rem;
		border-color: rgb(71 85 105);
		color: rgb(203 213 225);
	}
	
	:global(.wx-grid .wx-header-cell) {
		font-weight: 600;
		background: rgb(30 41 59);
		color: rgb(148 163 184);
		border-color: rgb(71 85 105);
		text-transform: uppercase;
		font-size: 0.7rem;
		letter-spacing: 0.05em;
		padding: 0.75rem 1rem;
	}
	
	:global(.wx-grid .wx-row:hover) {
		background: rgb(51 65 85);
	}
	
	:global(.wx-grid .wx-row:nth-child(even)) {
		background: rgb(15 23 42);
	}
	
	:global(.wx-grid .wx-row:nth-child(odd)) {
		background: rgb(15 23 42);
	}
	
	:global(.wx-grid .wx-pager) {
		background: rgb(30 41 59);
		border-top: 1px solid rgb(71 85 105);
		color: rgb(148 163 184);
	}
	
	:global(.wx-grid .wx-pager .wx-pager-button) {
		color: rgb(148 163 184);
		background: transparent;
		border: 1px solid rgb(71 85 105);
	}
	
	:global(.wx-grid .wx-pager .wx-pager-button:hover) {
		background: rgb(51 65 85);
		color: rgb(203 213 225);
	}
</style>