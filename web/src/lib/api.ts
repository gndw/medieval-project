import type {
  DateResponse,
  HomeResponse,
  PauseResponse,
} from "./types";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export async function fetchHome(): Promise<HomeResponse["data"]> {
  const res = await invoke<HomeResponse>("home");
  return res.data;
}

/** Fetch the current in-game date and pause state. */
export async function fetchDate(): Promise<DateResponse["data"]> {
  const res = await invoke<DateResponse>("get_date");
  return res.data;
}

/** Pause or resume the backend tick loop. Returns the applied state. */
export async function setPause(isPaused: boolean): Promise<boolean> {
  const res = await invoke<PauseResponse>("set_pause", {
    payload: { isPaused },
  });
  return res.data.is_paused;
}

/** Subscribe to the Rust-side `date-updated` event. Returns an unlisten fn. */
export function onDateUpdated(
  cb: (data: DateResponse["data"]) => void,
): Promise<UnlistenFn> {
  return listen<DateResponse["data"]>("date-updated", (e) => cb(e.payload));
}
