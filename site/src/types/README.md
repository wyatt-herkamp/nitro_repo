# Types

`api.d.ts` is **generated** — do not edit it.

Everything else here is hand-written against the Rust structs. The backend serves a utoipa OpenAPI
document, so those files can drift silently; `api.d.ts` cannot. Prefer it for new code, and move a
hand-written type over when you touch it.

To regenerate after a backend change:

```sh
npm run export-openapi     # dumps the spec from the server binary to openapi.json
npm run generate-api-types # openapi.json -> src/types/api.d.ts
```

`export-openapi` uses the `export open-api` subcommand rather than a running server, so this works
offline and in CI.
