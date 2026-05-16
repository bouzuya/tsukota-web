import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// https://vite.dev/config/
export default defineConfig({
	base: "/lab/tsukota",
	plugins: [react()],
	server: {
		host: "0.0.0.0",
		proxy: {
			"/lab/tsukota/api": {
				target: "http://localhost:3002",
				changeOrigin: false,
			},
			"/lab/tsukota/auth": {
				target: "http://localhost:3002",
				changeOrigin: false,
			},
		},
	},
});
