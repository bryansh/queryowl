<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { Button } from '$lib/components/ui/button';

	let showLogPath = $state(false);
	let logPath = $state('');

	onMount(() => {
		function handleKeydown(event: KeyboardEvent) {
			// Only handle when no input is focused
			if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
				return;
			}

			// Cmd+Shift+L (or Ctrl+Shift+L) - Open Log Folder
			if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key === 'L') {
				event.preventDefault();
				openLogFolder();
			}

			// Cmd+Shift+P (or Ctrl+Shift+P) - Show Log Path
			if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key === 'P') {
				event.preventDefault();
				showLogPathDialog();
			}
		}

		document.addEventListener('keydown', handleKeydown);

		return () => {
			document.removeEventListener('keydown', handleKeydown);
		};
	});

	async function openLogFolder() {
		try {
			await invoke('open_log_folder');
		} catch (error) {
			console.error('Failed to open log folder:', error);
		}
	}

	async function showLogPathDialog() {
		try {
			logPath = await invoke<string>('get_log_path');
			showLogPath = true;
		} catch (error) {
			console.error('Failed to get log path:', error);
		}
	}

	function copyLogPath() {
		navigator.clipboard.writeText(logPath);
		showLogPath = false;
	}
</script>

<!-- Log Path Dialog -->
{#if showLogPath}
	<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
		<div class="bg-card border rounded-lg p-6 max-w-lg mx-4">
			<h3 class="text-lg font-medium text-foreground mb-4">Log File Path</h3>
			<div class="bg-muted p-3 rounded text-sm font-mono break-all mb-4">
				{logPath}
			</div>
			<div class="flex gap-3">
				<Button onclick={copyLogPath}>
					Copy Path
				</Button>
				<Button variant="outline" onclick={() => showLogPath = false}>
					Close
				</Button>
			</div>
		</div>
	</div>
{/if}

<!-- Hidden shortcuts indicator (only show in dev) -->
{#if import.meta.env.DEV}
	<div class="fixed bottom-4 right-4 bg-card border rounded-lg p-2 text-xs text-muted-foreground opacity-75">
		Debug: ⌘⇧L (logs) · ⌘⇧P (path)
	</div>
{/if}