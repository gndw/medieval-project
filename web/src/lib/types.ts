/** Mirrors the JSON shape from Rust /api/v1/home. Keep in sync with src/http.rs. */

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
