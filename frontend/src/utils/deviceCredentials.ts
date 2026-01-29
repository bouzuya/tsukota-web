// Device ID (UUID v4) を生成
export function generateDeviceId(): string {
	return crypto.randomUUID();
}

// Device Secret (32バイトのランダム文字列) を生成
export function generateDeviceSecret(): string {
	const array = new Uint8Array(32);
	crypto.getRandomValues(array);
	return btoa(String.fromCharCode(...array));
}
