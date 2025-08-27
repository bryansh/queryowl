import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { DatabaseConnection, ConnectionStatus, CreateConnectionRequest, UpdateConnectionRequest, TestConnectionRequest, TestConnectionResponse } from '$lib/types/database';

export const connections = writable<DatabaseConnection[]>([]);
export const connectionStatus = writable<ConnectionStatus>({
	isConnected: false,
	activeConnection: undefined,
	error: undefined
});

export const activeConnection = derived(
	connectionStatus,
	($status) => $status.activeConnection
);

export const isConnected = derived(
	connectionStatus,
	($status) => $status.isConnected
);

export async function loadConnections(): Promise<void> {
	try {
		const stored = await invoke<DatabaseConnection[]>('get_stored_connections');
		connections.set(stored);
	} catch (error) {
		console.error('Failed to load connections:', error);
		connections.set([]);
	}
}

export async function saveConnection(connection: CreateConnectionRequest): Promise<DatabaseConnection> {
	try {
		const saved = await invoke<DatabaseConnection>('save_connection', { connection });
		connections.update(conns => [...conns, saved]);
		return saved;
	} catch (error) {
		console.error('Failed to save connection:', error);
		throw error;
	}
}

export async function updateConnection(connection: UpdateConnectionRequest): Promise<DatabaseConnection> {
	try {
		const updated = await invoke<DatabaseConnection>('update_connection', { connection });
		connections.update(conns => 
			conns.map(c => c.id === connection.id ? updated : c)
		);
		return updated;
	} catch (error) {
		console.error('Failed to update connection:', error);
		throw error;
	}
}

export async function deleteConnection(id: string): Promise<void> {
	try {
		await invoke('delete_connection', { id });
		connections.update(conns => conns.filter(c => c.id !== id));
		
		connectionStatus.update(status => {
			if (status.activeConnection?.id === id) {
				return { isConnected: false, activeConnection: undefined };
			}
			return status;
		});
	} catch (error) {
		console.error('Failed to delete connection:', error);
		throw error;
	}
}

export async function testConnection(connection: TestConnectionRequest): Promise<TestConnectionResponse> {
	try {
		return await invoke<TestConnectionResponse>('test_database_connection', { connection });
	} catch (error) {
		console.error('Failed to test connection:', error);
		return { success: false, error: String(error) };
	}
}

export async function connectToDatabase(connection: DatabaseConnection): Promise<void> {
	try {
		await invoke('connect_to_database', { connection });
		connectionStatus.set({
			isConnected: true,
			activeConnection: connection,
			error: undefined
		});
		
		await invoke('update_last_connected', { id: connection.id });
		connections.update(conns => 
			conns.map(c => c.id === connection.id ? { ...c, lastConnected: new Date().toISOString() } : c)
		);
	} catch (error) {
		console.error('Failed to connect to database:', error);
		connectionStatus.set({
			isConnected: false,
			activeConnection: undefined,
			error: String(error)
		});
		throw error;
	}
}

export async function disconnectFromDatabase(): Promise<void> {
	try {
		await invoke('disconnect_from_database');
		connectionStatus.set({
			isConnected: false,
			activeConnection: undefined,
			error: undefined
		});
	} catch (error) {
		console.error('Failed to disconnect from database:', error);
		throw error;
	}
}