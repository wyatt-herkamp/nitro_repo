// Generates `src/router/routes.json` from the router itself.
//
// The backend reads that file to decide which paths get the SPA's index.html rather than a 404
// (`nitro_repo/src/app/frontend/hosted.rs`), and a Rust test `include_str!`s it. It used to be
// maintained by hand alongside `router/index.ts`, with the only enforcement a dev-mode
// `console.error` in `App.vue` — which itself compared names with `=` instead of `===`, so it
// silently passed. A route missing from the file hard-404s on refresh in production.
//
// The router module is loaded through Vite's SSR pipeline so this stays a single source of truth
// rather than a parser guessing at the source.

import { createServer } from "vite";
import { fileURLToPath, URL } from "node:url";
import fs from "node:fs/promises";

const OUTPUT = fileURLToPath(new URL("../src/router/routes.json", import.meta.url));

/**
 * Replaces every `.vue` import with an empty module.
 *
 * Only `path` and `name` are read off each route, so the components are dead weight — and loading
 * them for real drags in highlight.js, Milkdown and FontAwesome, which touch `document` as they are
 * imported. Stubbing them keeps this to the route table and nothing else.
 */
const stubComponents = {
  name: "stub-vue-components",
  enforce: "pre",
  load(id) {
    if (id.endsWith(".vue")) {
      return "export default {}";
    }
  },
};

/**
 * A DOM stub just deep enough to import.
 *
 * `@vue/runtime-dom` builds a `<template>` element the moment it is required, and `config.ts` reads
 * `document.baseURI` at module scope. Nothing is rendered — these only have to exist.
 */
function stubBrowserGlobals() {
  const element = () => ({
    style: {},
    content: {},
    innerHTML: "",
    classList: { add() {}, remove() {} },
    setAttribute() {},
    appendChild() {},
    addEventListener() {},
  });

  // `createWebHistory` reads `window.history` and `window.location` while the router is being
  // constructed, which happens at import time.
  globalThis.window ??= {
    location: new URL("http://localhost/"),
    history: { state: null, replaceState() {}, pushState() {} },
    addEventListener() {},
    removeEventListener() {},
  };

  globalThis.document ??= {
    baseURI: "http://localhost/",
    createElement: element,
    createElementNS: element,
    createTextNode: element,
    createComment: element,
    querySelector: () => null,
    addEventListener() {},
    documentElement: element(),
    head: element(),
    body: element(),
  };
}

export async function generateRoutes({ check = false } = {}) {
  stubBrowserGlobals();

  const server = await createServer({
    // Without this, loading the config would re-enter the plugin that calls this script.
    configFile: false,
    logLevel: "error",
    server: { middlewareMode: true },
    // Nothing is served, so pre-bundling would only scan index.html and race the shutdown below.
    optimizeDeps: { noDiscovery: true, include: [] },
    resolve: { alias: { "@": fileURLToPath(new URL("../src", import.meta.url)) } },
    plugins: [stubComponents],
  });

  try {
    const { routes } = await server.ssrLoadModule("/src/router/routes.ts");

    const fallbackRoutes = routes
      // `skipRoutesJson` marks the routes the backend must not fall back for: `/` is served
      // directly, and the catch-all would swallow every genuine 404.
      .filter((route) => route.meta?.skipRoutesJson !== true)
      .map((route) => ({ path: route.path, name: route.name }));

    const rendered = `${JSON.stringify(fallbackRoutes, undefined, 2)}\n`;

    if (check) {
      const existing = await fs.readFile(OUTPUT, "utf8").catch(() => "");
      if (existing !== rendered) {
        throw new Error(
          "routes.json is out of date with router/routes.ts. Run `npm run generate-routes`.",
        );
      }
      return fallbackRoutes;
    }

    await fs.writeFile(OUTPUT, rendered);
    return fallbackRoutes;
  } finally {
    await server.close();
  }
}

// Allow running directly: `node scripts/generate-routes.mjs [--check]`.
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  generateRoutes({ check: process.argv.includes("--check") })
    .then((routes) => console.log(`routes.json: ${routes.length} routes`))
    .catch((error) => {
      console.error(error.message);
      process.exit(1);
    });
}
