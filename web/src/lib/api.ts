import type { DateResponse, GameDate, HomeResponse, Land, Road, Settlement } from "./types";

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

/** Fetch the current in-game date. Returns null while the world has no Date. */
export async function fetchDate(): Promise<GameDate | null> {
  const res = await fetch(`${BASE}/api/v1/date`);
  if (!res.ok) {
    throw new Error(`fetchDate failed: ${res.status} ${res.statusText}`);
  }
  const body = (await res.json()) as DateResponse;
  return body.data;
}
