import type { HomeResponse, Land } from "./types";

const BASE = ""; // empty in prod (same origin), proxied in dev

export async function fetchHome(): Promise<Land[]> {
  const res = await fetch(`${BASE}/api/v1/home`);
  if (!res.ok) {
    throw new Error(`fetchHome failed: ${res.status} ${res.statusText}`);
  }
  const body = (await res.json()) as HomeResponse;
  return body.data.lands;
}
