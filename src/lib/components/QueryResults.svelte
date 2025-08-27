<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Download, Copy, CheckCircle, AlertCircle, Table } from 'lucide-svelte';
	
	let { 
		results = [],
		error = null,
		executionTime = null,
		rowCount = 0
	}: {
		results?: any[];
		error?: string | null;
		executionTime?: number | null;
		rowCount?: number;
	} = $props();
	
	let copied = $state(false);
	
	// Compute row count reactively
	let actualRowCount = $derived(results.length);
	
	function copyToClipboard() {
		if (results.length === 0) return;
		
		const headers = Object.keys(results[0]);
		const csvContent = [
			headers.join('\t'),
			...results.map(row => headers.map(h => formatCellValue(row[h])).join('\t'))
		].join('\n');
		
		navigator.clipboard.writeText(csvContent);
		copied = true;
		setTimeout(() => copied = false, 2000);
	}
	
	function downloadCsv() {
		if (results.length === 0) return;
		
		const headers = Object.keys(results[0]);
		const csvContent = [
			headers.join(','),
			...results.map(row => 
				headers.map(h => {
					const val = formatCellValue(row[h]);
					// Escape values containing commas or quotes
					if (val.includes(',') || val.includes('"')) {
						return `"${val.replace(/"/g, '""')}"`;
					}
					return val;
				}).join(',')
			)
		].join('\n');
		
		const blob = new Blob([csvContent], { type: 'text/csv' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `query_results_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.csv`;
		a.click();
		URL.revokeObjectURL(url);
	}
	
	function formatCellValue(value: any): string {
		if (value === null) return 'NULL';
		if (value === undefined) return '';
		if (typeof value === 'object') {
			return JSON.stringify(value);
		}
		return String(value);
	}
	
	function getColumnType(values: any[]): string {
		const sample = values.find(v => v !== null && v !== undefined);
		if (sample === undefined) return 'null';
		
		if (typeof sample === 'number') return 'number';
		if (typeof sample === 'boolean') return 'boolean';
		if (sample instanceof Date) return 'date';
		if (typeof sample === 'object') return 'json';
		
		// Check if string looks like a date
		if (typeof sample === 'string' && !isNaN(Date.parse(sample))) {
			return 'date-string';
		}
		
		return 'string';
	}
</script>

<div class="query-results border rounded-lg bg-background">
	{#if error}
		<div class="p-4 bg-destructive/10 text-destructive rounded-t-lg border-b">
			<div class="flex items-center gap-2">
				<AlertCircle class="h-5 w-5" />
				<span class="font-medium">Query Error</span>
			</div>
			<pre class="mt-2 text-sm whitespace-pre-wrap font-mono">{error}</pre>
		</div>
	{:else if results.length > 0}
		<div class="results-header flex items-center justify-between p-3 border-b bg-muted/30">
			<div class="flex items-center gap-4 text-sm">
				<span class="flex items-center gap-1.5">
					<Table class="h-4 w-4" />
					<strong>{actualRowCount}</strong> {actualRowCount === 1 ? 'row' : 'rows'}
				</span>
				{#if executionTime !== null}
					<span class="text-muted-foreground">
						{executionTime}ms
					</span>
				{/if}
			</div>
			<div class="flex items-center gap-2">
				<Button 
					size="sm" 
					variant="outline"
					onclick={copyToClipboard}
				>
					{#if copied}
						<CheckCircle class="h-4 w-4 mr-2 text-green-500" />
						Copied!
					{:else}
						<Copy class="h-4 w-4 mr-2" />
						Copy
					{/if}
				</Button>
				<Button 
					size="sm" 
					variant="outline"
					onclick={downloadCsv}
				>
					<Download class="h-4 w-4 mr-2" />
					Export CSV
				</Button>
			</div>
		</div>
		
		<div class="results-table-container overflow-auto max-h-[500px]">
			<table class="results-table w-full">
				<thead class="sticky top-0 bg-muted/50 backdrop-blur-sm">
					<tr>
						{#each Object.keys(results[0]) as column}
							<th class="px-4 py-2 text-left text-sm font-medium text-foreground border-b">
								{column}
								<span class="ml-2 text-xs text-muted-foreground">
									{getColumnType(results.map(r => r[column]))}
								</span>
							</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each results as row, i}
						<tr class="hover:bg-muted/30 transition-colors">
							{#each Object.keys(results[0]) as column}
								{@const value = row[column]}
								{@const type = getColumnType([value])}
								<td class="px-4 py-2 text-sm border-b font-mono {type === 'number' ? 'text-right' : ''}">
									{#if value === null}
										<span class="text-muted-foreground italic">NULL</span>
									{:else if type === 'boolean'}
										<span class="font-semibold {value ? 'text-green-600' : 'text-red-600'}">
											{value}
										</span>
									{:else if type === 'json'}
										<pre class="text-xs bg-muted p-1 rounded max-w-xs overflow-auto">{JSON.stringify(value, null, 2)}</pre>
									{:else if type === 'date' || type === 'date-string'}
										<span class="text-blue-600 dark:text-blue-400">
											{new Date(value).toLocaleString()}
										</span>
									{:else}
										{formatCellValue(value)}
									{/if}
								</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<div class="p-8 text-center text-muted-foreground">
			<Table class="h-12 w-12 mx-auto mb-3 opacity-50" />
			<p>No results to display</p>
			<p class="text-sm mt-1">Run a query to see results here</p>
		</div>
	{/if}
</div>

<style>
	.query-results {
		width: 100%;
	}
	
	.results-table-container {
		position: relative;
	}
	
	.results-table {
		border-collapse: collapse;
		table-layout: auto;
	}
	
	.results-table th {
		white-space: nowrap;
		position: sticky;
		top: 0;
		z-index: 10;
	}
	
	.results-table td {
		max-width: 400px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	
	.results-table td:hover {
		white-space: normal;
		word-break: break-word;
	}
</style>