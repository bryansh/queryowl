<script lang="ts">
	import { FileDown, X, AlertCircle } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { save } from '@tauri-apps/plugin-dialog';
	
	let {
		show = $bindable(false),
		sql,
		results,
		metadata,
		connectionId,
		onClose
	}: {
		show?: boolean;
		sql: string;
		results: any[];
		metadata?: any;
		connectionId: string;
		onClose?: () => void;
	} = $props();
	
	let exportFormat = $state('csv');
	let exportScope = $state('current'); // 'current' or 'all'
	let includeHeaders = $state(true);
	let quoteAllValues = $state(false);
	let useNativeCopy = $state(false); // Use PostgreSQL COPY TO
	let exporting = $state(false);
	let exportProgress = $state('');
	
	// Calculate row counts
	let currentRows = $derived(results?.length || 0);
	let totalRows = $derived(metadata?.total_rows || currentRows);
	let hasMoreRows = $derived(metadata?.limit_applied || false);
	
	async function handleExport() {
		if (!connectionId || !sql) return;
		
		exporting = true;
		exportProgress = 'Preparing export...';
		
		try {
			// Determine export method based on size and options
			if (exportScope === 'all' && totalRows > 10000 && useNativeCopy && exportFormat === 'csv') {
				// Use PostgreSQL COPY TO for large CSV exports
				await exportWithCopyTo();
			} else if (exportScope === 'all' && totalRows > currentRows) {
				// Stream from backend for all rows
				await exportStreamFromBackend();
			} else {
				// Export current view from frontend
				await exportCurrentView();
			}
		} catch (error) {
			console.error('Export failed:', error);
			alert(`Export failed: ${error}`);
		} finally {
			exporting = false;
			exportProgress = '';
		}
	}
	
	async function exportWithCopyTo() {
		exportProgress = 'Using PostgreSQL COPY TO for maximum performance...';
		
		// Prompt user for save location
		const filePath = await save({
			filters: [{
				name: 'CSV',
				extensions: ['csv']
			}],
			defaultPath: `export_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.csv`
		});
		
		if (!filePath) {
			exporting = false;
			return;
		}
		
		// Call backend to execute COPY TO
		const result = await invoke('export_query_native', {
			connectionId,
			sql,
			outputPath: filePath,
			format: 'csv',
			includeHeaders
		});
		
		exportProgress = 'Export complete!';
		setTimeout(() => {
			show = false;
			if (onClose) onClose();
		}, 1000);
	}
	
	async function exportStreamFromBackend() {
		exportProgress = `Streaming ${totalRows.toLocaleString()} rows from database...`;
		
		// Prompt user for save location
		const filePath = await save({
			filters: exportFormat === 'csv' ? 
				[{ name: 'CSV', extensions: ['csv'] }] : 
				[{ name: 'JSON', extensions: ['json'] }],
			defaultPath: `export_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.${exportFormat}`
		});
		
		if (!filePath) {
			exporting = false;
			return;
		}
		
		// Call backend to stream results to file
		const result = await invoke('export_query_stream', {
			connectionId,
			sql,
			outputPath: filePath,
			format: exportFormat,
			options: {
				includeHeaders,
				quoteAllValues
			}
		});
		
		exportProgress = 'Export complete!';
		setTimeout(() => {
			show = false;
			if (onClose) onClose();
		}, 1000);
	}
	
	async function exportCurrentView() {
		exportProgress = `Exporting ${currentRows.toLocaleString()} rows...`;
		
		if (exportFormat === 'csv') {
			exportCurrentAsCSV();
		} else {
			exportCurrentAsJSON();
		}
		
		exportProgress = 'Export complete!';
		setTimeout(() => {
			show = false;
			if (onClose) onClose();
		}, 1000);
	}
	
	function exportCurrentAsCSV() {
		const headers = Object.keys(results[0]);
		const rows = results.map(row => 
			headers.map(header => {
				const value = row[header];
				if (value === null) return 'NULL';
				if (typeof value === 'object') return JSON.stringify(value);
				const strValue = String(value);
				
				if (quoteAllValues || strValue.includes(',') || strValue.includes('"') || strValue.includes('\n')) {
					return `"${strValue.replace(/"/g, '""')}"`;
				}
				return strValue;
			}).join(',')
		);
		
		const csvContent = includeHeaders ? 
			[headers.join(','), ...rows].join('\n') : 
			rows.join('\n');
			
		downloadFile(csvContent, `export_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.csv`, 'text/csv');
	}
	
	function exportCurrentAsJSON() {
		const jsonContent = JSON.stringify(results, null, 2);
		downloadFile(jsonContent, `export_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.json`, 'application/json');
	}
	
	function downloadFile(content: string, filename: string, mimeType: string) {
		const blob = new Blob([content], { type: mimeType });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = filename;
		a.click();
		URL.revokeObjectURL(url);
	}
	
	function handleClose() {
		if (!exporting) {
			show = false;
			if (onClose) onClose();
		}
	}
</script>

{#if show}
	<div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
		<div class="card p-6 max-w-md w-full mx-4 shadow-2xl">
			<div class="flex items-center justify-between mb-4">
				<h3 class="text-xl font-semibold flex items-center gap-2">
					<FileDown class="h-5 w-5" />
					Export Results
				</h3>
				<button 
					onclick={handleClose}
					disabled={exporting}
					class="hover:bg-surface-300-700 rounded p-1 transition-colors"
				>
					<X class="h-5 w-5" />
				</button>
			</div>
			
			{#if !exporting}
				<div class="space-y-4">
					<!-- Format Selection -->
					<div>
						<label class="block text-sm font-medium mb-2">Export Format</label>
						<div class="flex gap-3">
							<label class="flex items-center">
								<input 
									type="radio" 
									bind:group={exportFormat} 
									value="csv" 
									class="mr-2"
								/>
								CSV
							</label>
							<label class="flex items-center">
								<input 
									type="radio" 
									bind:group={exportFormat} 
									value="json" 
									class="mr-2"
								/>
								JSON
							</label>
						</div>
					</div>
					
					<!-- Rows Selection -->
					<div>
						<label class="block text-sm font-medium mb-2">Rows to Export</label>
						<div class="space-y-2">
							<label class="flex items-center">
								<input 
									type="radio" 
									bind:group={exportScope} 
									value="current" 
									class="mr-2"
								/>
								Current view ({currentRows.toLocaleString()} rows)
							</label>
							{#if hasMoreRows}
								<label class="flex items-center">
									<input 
										type="radio" 
										bind:group={exportScope} 
										value="all" 
										class="mr-2"
									/>
									All rows ({totalRows.toLocaleString()} rows)
									{#if totalRows > 50000}
										<span class="ml-2 text-yellow-500 text-xs">(Large dataset)</span>
									{/if}
								</label>
							{/if}
						</div>
					</div>
					
					<!-- CSV Options -->
					{#if exportFormat === 'csv'}
						<div class="space-y-2">
							<label class="flex items-center">
								<input 
									type="checkbox" 
									bind:checked={includeHeaders} 
									class="mr-2"
								/>
								Include column headers
							</label>
							<label class="flex items-center">
								<input 
									type="checkbox" 
									bind:checked={quoteAllValues} 
									class="mr-2"
								/>
								Quote all values
							</label>
							{#if exportScope === 'all' && totalRows > 10000}
								<label class="flex items-center">
									<input 
										type="checkbox" 
										bind:checked={useNativeCopy} 
										class="mr-2"
									/>
									Use PostgreSQL COPY (fastest)
								</label>
							{/if}
						</div>
					{/if}
					
					<!-- Warning for large exports -->
					{#if exportScope === 'all' && totalRows > 100000}
						<div class="p-3 bg-yellow-900/20 text-yellow-400 rounded-lg flex items-start gap-2">
							<AlertCircle class="h-5 w-5 flex-shrink-0 mt-0.5" />
							<div class="text-sm">
								<p class="font-medium">Large Export Warning</p>
								<p class="mt-1 opacity-80">
									Exporting {totalRows.toLocaleString()} rows may take several minutes.
									{#if exportFormat === 'csv' && !useNativeCopy}
										Consider enabling "Use PostgreSQL COPY" for better performance.
									{/if}
								</p>
							</div>
						</div>
					{/if}
				</div>
				
				<div class="flex gap-3 mt-6">
					<button
						onclick={handleExport}
						class="btn btn-filled-primary flex-1"
					>
						<FileDown class="h-4 w-4 mr-2" />
						Export
					</button>
					<button
						onclick={handleClose}
						class="btn btn-ghost-surface"
					>
						Cancel
					</button>
				</div>
			{:else}
				<!-- Export Progress -->
				<div class="py-8 text-center">
					<div class="inline-flex items-center justify-center w-12 h-12 rounded-full bg-primary-500/20 mb-4">
						<FileDown class="h-6 w-6 text-primary-500 animate-pulse" />
					</div>
					<p class="text-lg font-medium mb-2">Exporting...</p>
					<p class="text-sm text-surface-400">{exportProgress}</p>
				</div>
			{/if}
		</div>
	</div>
{/if}