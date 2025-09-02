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
			
			// Export completed successfully
			exportProgress = 'Export complete!';
			
			// Close dialog after a brief success message
			await new Promise(resolve => setTimeout(resolve, 500));
			show = false;
			if (onClose) onClose();
		} catch (error) {
			console.error('Export failed:', error);
			exportProgress = `Export failed: ${error}`;
			// Keep dialog open on error so user can see the message
			exporting = false;
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
			throw new Error('Export cancelled by user');
		}
		
		// Call backend to execute COPY TO
		const result = await invoke('export_query_native', {
			connectionId,
			sql,
			outputPath: filePath,
			format: 'csv',
			includeHeaders
		});
		
		exportProgress = `Export complete! Saved to ${filePath.split('/').pop()}`;
		return result;
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
			throw new Error('Export cancelled by user');
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
		
		exportProgress = `Export complete! Saved to ${filePath.split('/').pop()}`;
		return result;
	}
	
	async function exportCurrentView() {
		exportProgress = `Exporting ${currentRows.toLocaleString()} rows...`;
		
		if (exportFormat === 'csv') {
			await exportCurrentAsCSV();
		} else {
			await exportCurrentAsJSON();
		}
		
		exportProgress = 'Export complete! File saved to your Downloads folder.';
		return 'Browser download completed';
	}
	
	async function exportCurrentAsCSV() {
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
		
		// For current view, use browser download (simpler for small files)
		downloadFile(csvContent, `export_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.csv`, 'text/csv');
	}
	
	async function exportCurrentAsJSON() {
		const jsonContent = JSON.stringify(results, null, 2);
		// For current view, use browser download (simpler for small files)
		downloadFile(jsonContent, `export_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.json`, 'application/json');
	}
	
	function downloadFile(content: string, filename: string, mimeType: string) {
		// This creates a download in the browser's default download folder
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
	<div class="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50">
		<div class="bg-surface-800 border border-surface-600 rounded-xl p-6 max-w-md w-full mx-4 shadow-2xl">
			<div class="flex items-center justify-between mb-6">
				<h3 class="text-2xl font-semibold flex items-center gap-3 text-surface-100">
					<div class="p-2 bg-primary-500/20 rounded-lg">
						<FileDown class="h-5 w-5 text-primary-400" />
					</div>
					Export Results
				</h3>
				<button 
					onclick={handleClose}
					disabled={exporting}
					class="hover:bg-surface-700 rounded-lg p-2 transition-colors text-surface-400 hover:text-surface-200"
				>
					<X class="h-5 w-5" />
				</button>
			</div>
			
			{#if !exporting}
				<div class="space-y-5">
					<!-- Format Selection -->
					<div>
						<label class="block text-sm font-medium mb-3 text-surface-300">Export Format</label>
						<div class="grid grid-cols-2 gap-3">
							<label class="relative flex items-center justify-center p-3 rounded-lg border-2 transition-all cursor-pointer {exportFormat === 'csv' ? 'bg-primary-500/10 border-primary-500 text-primary-400' : 'bg-surface-700 border-surface-600 hover:border-surface-500 text-surface-300'}">
								<input 
									type="radio" 
									bind:group={exportFormat} 
									value="csv" 
									class="sr-only"
								/>
								<span class="font-medium">CSV</span>
							</label>
							<label class="relative flex items-center justify-center p-3 rounded-lg border-2 transition-all cursor-pointer {exportFormat === 'json' ? 'bg-primary-500/10 border-primary-500 text-primary-400' : 'bg-surface-700 border-surface-600 hover:border-surface-500 text-surface-300'}">
								<input 
									type="radio" 
									bind:group={exportFormat} 
									value="json" 
									class="sr-only"
								/>
								<span class="font-medium">JSON</span>
							</label>
						</div>
					</div>
					
					<!-- Rows Selection -->
					<div>
						<label class="block text-sm font-medium mb-3 text-surface-300">Rows to Export</label>
						<div class="space-y-2">
							<label class="flex items-center p-3 rounded-lg bg-surface-700 hover:bg-surface-600 transition-colors cursor-pointer {exportScope === 'current' ? 'ring-2 ring-primary-500' : ''}">
								<input 
									type="radio" 
									bind:group={exportScope} 
									value="current" 
									class="mr-3 text-primary-500 focus:ring-primary-500"
								/>
								<div class="flex-1">
									<span class="text-surface-100 font-medium">Current view</span>
									<span class="text-surface-400 text-sm ml-2">({currentRows.toLocaleString()} rows)</span>
									<div class="text-xs text-surface-500 mt-1">→ Downloads folder</div>
								</div>
							</label>
							{#if hasMoreRows}
								<label class="flex items-center p-3 rounded-lg bg-surface-700 hover:bg-surface-600 transition-colors cursor-pointer {exportScope === 'all' ? 'ring-2 ring-primary-500' : ''}">
									<input 
										type="radio" 
										bind:group={exportScope} 
										value="all" 
										class="mr-3 text-primary-500 focus:ring-primary-500"
									/>
									<div class="flex-1">
										<span class="text-surface-100 font-medium">All rows</span>
										<span class="text-surface-400 text-sm ml-2">({totalRows.toLocaleString()} rows)</span>
										{#if totalRows > 50000}
											<span class="ml-2 text-yellow-400 text-xs font-medium bg-yellow-500/10 px-2 py-0.5 rounded">Large dataset</span>
										{/if}
										<div class="text-xs text-surface-500 mt-1">→ Choose save location</div>
									</div>
								</label>
							{/if}
						</div>
					</div>
					
					<!-- CSV Options -->
					{#if exportFormat === 'csv'}
						<div class="bg-surface-700/50 rounded-lg p-4 space-y-3">
							<h4 class="text-sm font-medium text-surface-300 mb-2">CSV Options</h4>
							<label class="flex items-center cursor-pointer hover:text-surface-100 transition-colors">
								<input 
									type="checkbox" 
									bind:checked={includeHeaders} 
									class="mr-3 rounded border-surface-500 bg-surface-600 text-primary-500 focus:ring-primary-500"
								/>
								<span class="text-surface-200">Include column headers</span>
							</label>
							<label class="flex items-center cursor-pointer hover:text-surface-100 transition-colors">
								<input 
									type="checkbox" 
									bind:checked={quoteAllValues} 
									class="mr-3 rounded border-surface-500 bg-surface-600 text-primary-500 focus:ring-primary-500"
								/>
								<span class="text-surface-200">Quote all values</span>
							</label>
							{#if exportScope === 'all' && totalRows > 10000}
								<label class="flex items-center cursor-pointer hover:text-surface-100 transition-colors">
									<input 
										type="checkbox" 
										bind:checked={useNativeCopy} 
										class="mr-3 rounded border-surface-500 bg-surface-600 text-primary-500 focus:ring-primary-500"
									/>
									<div class="flex items-center gap-2">
										<span class="text-surface-200">Use PostgreSQL COPY</span>
										<span class="text-xs bg-green-500/20 text-green-400 px-2 py-0.5 rounded font-medium">Fastest</span>
									</div>
								</label>
							{/if}
						</div>
					{/if}
					
					<!-- Warning for large exports -->
					{#if exportScope === 'all' && totalRows > 100000}
						<div class="p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg flex items-start gap-3">
							<AlertCircle class="h-5 w-5 flex-shrink-0 mt-0.5 text-yellow-400" />
							<div class="text-sm">
								<p class="font-medium text-yellow-400">Large Export Warning</p>
								<p class="mt-1 text-yellow-400/80">
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
						class="flex-1 bg-primary-600 hover:bg-primary-700 text-white rounded-lg px-4 py-3 font-medium transition-colors flex items-center justify-center gap-2"
					>
						<FileDown class="h-5 w-5" />
						Export
					</button>
					<button
						onclick={handleClose}
						class="px-4 py-3 rounded-lg bg-surface-700 hover:bg-surface-600 text-surface-200 font-medium transition-colors"
					>
						Cancel
					</button>
				</div>
			{:else}
				<!-- Export Progress -->
				<div class="py-12 text-center">
					<div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-primary-500/20 mb-4">
						<FileDown class="h-8 w-8 text-primary-400 animate-pulse" />
					</div>
					<p class="text-xl font-medium mb-2 text-surface-100">Exporting...</p>
					<p class="text-sm text-surface-400">{exportProgress}</p>
					<div class="mt-6 h-2 bg-surface-700 rounded-full overflow-hidden">
						<div class="h-full bg-primary-500 rounded-full animate-pulse" style="width: 60%"></div>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}