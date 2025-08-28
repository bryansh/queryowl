export interface DatabaseConnection {
	id: string;
	name: string;
	host: string;
	port: number;
	database: string;
	username: string;
	password?: string;
	ssl?: boolean;
	createdAt: string;
	lastConnected?: string;
	color?: string;
}

export interface ConnectionStatus {
	isConnected: boolean;
	activeConnection?: DatabaseConnection;
	error?: string;
}

export interface CreateConnectionRequest {
	name: string;
	host: string;
	port: number;
	database: string;
	username: string;
	password: string;
	ssl?: boolean;
	color?: string;
}

export interface UpdateConnectionRequest {
	id: string;
	name: string;
	host: string;
	port: number;
	database: string;
	username: string;
	password: string;
	ssl?: boolean;
	color?: string;
}

export interface TestConnectionRequest {
	host: string;
	port: number;
	database: string;
	username: string;
	password: string;
	ssl?: boolean;
}

export interface TestConnectionResponse {
	success: boolean;
	error?: string;
}