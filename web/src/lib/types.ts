/**
 * Mirrors the JSON shape returned by the Rust backend at /api/v1/home.
 * Keep this in sync with src/http.rs in the backend.
 */

export type Terrain = "plains" | "wetlands" | "forest";

export interface Land {
  id: string;
  name: string;
  terrain: Terrain;
  /** [x, y] position of the holding. */
  holding: [number, number];
  /** Polygon vertices forming the land's border. */
  borders: Array<[number, number]>;
}

export interface Road {
  id: string;
  /** Ordered waypoints along the road's path. */
  points: Array<[number, number]>;
  /** String IDs of the lands this road connects. */
  between_land_ids: string[];
  /** Travel time in in-game days. */
  distance_days: number;
}

export interface Settlement {
  id: string;
  /** String ID of the land this settlement belongs to. */
  land_id: string;
  population: number;
}

export interface HomeResponse {
  data: {
    lands: Land[];
    roads: Road[];
    settlements: Settlement[];
  };
}
