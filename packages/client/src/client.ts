import type { ClientOptions } from "openapi-fetch";
import createFetchClientImpl from "openapi-fetch";

import type { paths as APIPaths } from "./types.ts";

export function createFetchClient(params: ClientOptions) {
  return createFetchClientImpl<APIPaths>(params);
}

export type FetchClient = ReturnType<typeof createFetchClient>;
