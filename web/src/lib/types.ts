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

export interface HomeResponse {
  data: {
    lands: Land[];
  };
}
