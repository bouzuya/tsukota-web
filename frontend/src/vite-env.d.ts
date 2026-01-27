/// <reference types="vite/client" />

interface ImportMetaEnv {
	/** API のベース URL (開発時: "http://localhost:3000", 本番: "") */
	readonly VITE_API_BASE: string;
}

interface ImportMeta {
	readonly env: ImportMetaEnv;
}
