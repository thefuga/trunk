import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { showToast } from "./toast.svelte.js";

/** Copy a commit SHA to the clipboard and confirm via toast.
 *  Always copies the full oid, even when only a short form is shown on screen. */
export async function copySha(oid: string): Promise<void> {
	try {
		await writeText(oid);
		showToast(`Copied ${oid.slice(0, 7)}`);
	} catch (err) {
		const message = err instanceof Error ? err.message : String(err);
		showToast(`Failed to copy: ${message}`, "error");
	}
}
