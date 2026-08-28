import { writable } from "svelte/store";
import type { Land } from "./types";

/** All lands currently loaded from the backend. */
export const lands = writable<Land[]>([]);

/** The currently selected land, or null if nothing is selected. */
export const selectedLandId = writable<string | null>(null);

/** True while the initial fetch is in flight. */
export const loading = writable<boolean>(true);

/** Set if the last fetch failed. */
export const error = writable<string | null>(null);
