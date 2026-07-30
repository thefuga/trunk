import { safeInvoke } from "./invoke.js";
import type { WslAvailability, WslDistro } from "./types.js";

export interface WslOpenTargets {
	availability: WslAvailability | null;
	distros: WslDistro[];
}

// Availability probe + distro list for the "Open Repository" dropdown.
// Never throws: WSL problems degrade to a plain local-open button.
export async function loadWslOpenTargets(): Promise<WslOpenTargets> {
	const availability = await safeInvoke<WslAvailability>(
		"wsl_availability",
	).catch(() => null);
	if (!availability?.available) return { availability, distros: [] };

	const distros = await safeInvoke<WslDistro[]>("list_wsl_distros").catch(
		() => [],
	);
	return { availability, distros };
}

export function sortDistrosDefaultFirst(distros: WslDistro[]): WslDistro[] {
	return [...distros].sort((a, b) => Number(b.default) - Number(a.default));
}
