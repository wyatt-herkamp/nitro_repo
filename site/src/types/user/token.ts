import type { RepositoryActions } from "../base";
import type { RepositoryActionsType } from "../user";

export interface RawAuthTokenResponse {
  id: number;
  user_id: number;
  name?: string;
  description?: string;
  active: boolean;
  source: string;
  expires_at?: string;
  /** When the token last authenticated a request. Recorded at most once an hour. */
  last_used_at?: string;
  created_at: string;
}

export interface RawAuthTokenFullResponse {
  token: RawAuthTokenResponse;
  scopes: Array<RawAuthTokenScopes>;
  repository_scopes: Array<RawAuthTokenRepositoryScope>;
}

export interface RawAuthTokenScopes {
  id: number;
  user_auth_token_id: number;
  scope: string;
}
export interface RawAuthTokenRepositoryScope {
  id: number;
  user_auth_token_id: number;
  repository_id: number;
  actions: Array<RepositoryActions>;
}

export interface NewAuthTokenResponse {
  id: number;
  token: string;
}

export interface NewAuthTokenRepositoryScope {
  repositoryId: string;
  actions: RepositoryActionsType;
}

/**
 * The shape the API actually accepts for a repository scope.
 *
 * The create form was sending `{ repository_string, actions }`, which has neither field the server
 * looks for, so any token created with a repository scope was rejected as a bad request.
 */
export interface NewAuthTokenRepositoryScopeRequest {
  repository_id: string;
  scopes: Array<RepositoryActions>;
}
