import { homeDir } from "@tauri-apps/api/path";

const home = await homeDir().catch(() => "");

export function displayPath(path: string): string {
	if (!home || !path.startsWith(home)) return path;
	const rest = path.slice(home.length).replace(/^\//, "");
	return `~/${rest}`;
}
