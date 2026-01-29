/// <reference types="vite/client" />

interface ImportMetaEnv {
	/** API のベース URL */
	readonly VITE_API_BASE: string;
}

interface ImportMeta {
	readonly env: ImportMetaEnv;
}
