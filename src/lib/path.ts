import { homeDir } from "@tauri-apps/api/path";

let home: string | undefined;

async function getHome(): Promise<string> {
	if (home === undefined) {
		home = await homeDir().catch(() => "");
	}
	return home;
}

export async function displayPath(path: string): Promise<string> {
	const h = await getHome();
	if (!h || !path.startsWith(h)) return path;
	const rest = path.slice(h.length).replace(/^\//, "");
	return `~/${rest}`;
}
