export interface GeneratedRoute {
  path: string;
  name: string;
}

/**
 * Writes `src/router/routes.json` from the route table. With `check`, throws instead of writing when
 * the file on disk does not match.
 */
export function generateRoutes(options?: { check?: boolean }): Promise<Array<GeneratedRoute>>;
