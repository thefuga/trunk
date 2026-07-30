// Parsing of WSL UNC paths (\\wsl.localhost\<Distro>\<path>) as returned by
// the native folder picker. The inverse lives in Rust: commands::wsl::unc_path.

export interface WslUncTarget {
	distro: string;
	linuxPath: string;
}

const WSL_UNC_PATTERN = /^\\\\(?:wsl\.localhost|wsl\$)\\([^\\]+)(?:\\(.*))?$/i;

export function parseWslUncPath(path: string): WslUncTarget | null {
	const normalized = path.replace(/\//g, "\\");
	const match = normalized.match(WSL_UNC_PATTERN);
	if (!match?.[1]) return null;

	const distro = match[1];
	const rest = (match[2] ?? "").replace(/\\+$/, "");
	const linuxPath = rest.length === 0 ? "/" : `/${rest.replace(/\\+/g, "/")}`;
	return { distro, linuxPath };
}

export function wslRootUncPath(distro: string): string {
	return `\\\\wsl.localhost\\${distro}`;
}
