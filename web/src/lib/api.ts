import type {
  DateResponse,
  GameDate,
  HomeResponse,
  Land,
  PauseResponse,
  Road,
  Settlement,
} from "./types";

const BASE = ""; // empty in prod (same origin), proxied in dev

export async function fetchHome(): Promise<{
  lands: Land[];
  roads: Road[];
  settlements: Settlement[];
}> {
  const res = await fetch(`${BASE}/api/v1/home`);
  if (!res.ok) {
    throw new Error(`fetchHome failed: ${res.status} ${res.statusText}`);
  }
  const body = (await res.json()) as HomeResponse;
  return body.data;
}

/** Fetch the current in-game date and pause state. */
export async function fetchDate(): Promise<{
  date: GameDate | null;
  is_paused: boolean;
}> {
  const res = await fetch(`${BASE}/api/v1/date`);
  if (!res.ok) {
    throw new Error(`fetchDate failed: ${res.status} ${res.statusText}`);
  }
  const body = (await res.json()) as DateResponse;
  return body.data;
}

/** Pause or resume the backend tick loop. Returns the applied state. */
export async function setPause(isPaused: boolean): Promise<boolean> {
  const res = await fetch(`${BASE}/api/v1/date`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ is_paused: isPaused }),
  });
  if (!res.ok) {
    throw new Error(`setPause failed: ${res.status} ${res.statusText}`);
  }
  const body = (await res.json()) as PauseResponse;
  return body.data.is_paused;
}
