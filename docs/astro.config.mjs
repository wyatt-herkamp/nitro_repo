// @ts-check
import sitemap from "@astrojs/sitemap";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import starlightLinksValidator from "starlight-links-validator";

const GITHUB = "https://github.com/wyatt-herkamp/nitro_repo";

export default defineConfig({
  site: "https://nitro-repo.kingtux.dev",
  // GitHub Pages serves the built directory verbatim, and Starlight's own links are all absolute
  // from the site root, so a trailing-slash mismatch shows up as a redirect on every internal
  // navigation. Pinning it means the sitemap, the canonical tags and the anchors all agree.
  trailingSlash: "always",
  // Anything written against the VitePress site is still in the wild — the repository-type pages in
  // particular, because the server hands those URLs to the frontend as a repository type's
  // `documentation_url`.
  //
  // `/knowledge/InternalWorkings` and `/sysAdmin/` described a 1.x that no longer exists — MySQL,
  // a `--install` flag, a systemd unit shipped inside the tarball. They point at the pages that
  // replaced them rather than being restored.
  redirects: {
    "/features/": "/about/comparison/",
    "/contributing/": "/develop/",
    "/sysAdmin/": "/start/installation/",
    "/knowledge/": "/about/architecture/",
    "/knowledge/InternalWorkings/": "/about/architecture/",
    "/repositoryTypes/": "/repositories/types/",
    "/repositoryTypes/maven/": "/repositories/maven/",
    "/repositoryTypes/maven/standard/": "/repositories/maven/deploying/",
    "/repositoryTypes/maven/configs/": "/repositories/maven/configuration/",
    "/repositoryTypes/maven/nitroDeploy/": "/repositories/maven/",
    "/repositoryTypes/npm/": "/repositories/npm/",
    "/repositoryTypes/npm/standard/": "/repositories/npm/publishing/",
    "/repositoryTypes/npm/configs/": "/repositories/npm/configuration/",
    "/repositoryTypes/npm/errors/": "/repositories/npm/troubleshooting/",
  },
  integrations: [
    starlight({
      title: "Nitro Repo",
      description:
        "An open source artifact manager for Maven and npm, with a Rust backend and a Vue frontend.",
      // The application's own icon, copied from `site/public/`. The three sizes and the order they
      // are declared in mirror `site/index.html`, so a tab open on the docs and a tab open on an
      // instance show the same mark rather than two things that nearly match.
      favicon: "/icon.png",
      head: [
        {
          tag: "link",
          attrs: { rel: "icon", href: "/icon-64.png", sizes: "64x64" },
        },
        {
          tag: "link",
          attrs: { rel: "icon", href: "/icon-128.png", sizes: "128x128" },
        },
      ],
      // The docs and the application should read as one product, so they share a palette rather
      // than each having its own.
      customCss: [
        "@fontsource/ibm-plex-sans/400.css",
        "@fontsource/ibm-plex-sans/500.css",
        "@fontsource/ibm-plex-sans/600.css",
        "@fontsource/ibm-plex-mono/400.css",
        "@fontsource/ibm-plex-mono/500.css",
        "./src/styles/theme.css",
      ],
      // Fails the build on a link to a page or anchor that does not exist. Docs rot by way of
      // links long before they rot by way of prose, and a broken link is the one kind of rot a
      // machine can catch.
      plugins: [starlightLinksValidator({ errorOnRelativeLinks: false })],
      social: [{ icon: "github", label: "GitHub", href: GITHUB }],
      editLink: {
        baseUrl: `${GITHUB}/edit/main/docs/`,
      },
      lastUpdated: true,
      credits: false,
      tableOfContents: { minHeadingLevel: 2, maxHeadingLevel: 3 },
      sidebar: [
        {
          label: "Getting started",
          items: [
            { label: "What is Nitro Repo?", slug: "start" },
            { label: "Installation", slug: "start/installation" },
            { label: "First run", slug: "start/first-run" },
            { label: "Core concepts", slug: "start/concepts" },
          ],
        },
        {
          label: "Repositories",
          items: [
            { label: "Repository types", slug: "repositories/types" },
            {
              label: "Maven",
              collapsed: false,
              items: [
                { label: "Overview", slug: "repositories/maven" },
                {
                  label: "Deploying and resolving",
                  slug: "repositories/maven/deploying",
                },
                {
                  label: "Proxy repositories",
                  slug: "repositories/maven/proxy",
                },
                {
                  label: "Configuration",
                  slug: "repositories/maven/configuration",
                },
              ],
            },
            {
              label: "npm",
              collapsed: false,
              items: [
                { label: "Overview", slug: "repositories/npm" },
                {
                  label: "Authenticating",
                  slug: "repositories/npm/authenticating",
                },
                { label: "Publishing", slug: "repositories/npm/publishing" },
                {
                  label: "Configuration",
                  slug: "repositories/npm/configuration",
                },
                {
                  label: "Troubleshooting",
                  slug: "repositories/npm/troubleshooting",
                },
              ],
            },
            {
              label: "Cargo",
              collapsed: false,
              items: [
                { label: "Overview", slug: "repositories/cargo" },
                {
                  label: "Authenticating",
                  slug: "repositories/cargo/authenticating",
                },
                { label: "Publishing", slug: "repositories/cargo/publishing" },
                {
                  label: "Configuration",
                  slug: "repositories/cargo/configuration",
                },
              ],
            },
            {
              label: "Docker",
              collapsed: false,
              items: [
                { label: "Overview", slug: "repositories/docker" },
                {
                  label: "Authenticating",
                  slug: "repositories/docker/authenticating",
                },
                {
                  label: "Pushing and pulling",
                  slug: "repositories/docker/pushing",
                },
                {
                  label: "Configuration",
                  slug: "repositories/docker/configuration",
                },
              ],
            },
          ],
        },
        {
          label: "Administration",
          items: [
            { label: "Storages", slug: "admin/storages" },
            { label: "Custom domains", slug: "admin/custom-domains" },
            { label: "Users and permissions", slug: "admin/users" },
            { label: "API tokens and scopes", slug: "admin/tokens" },
            { label: "Search", slug: "admin/search" },
            { label: "Badges", slug: "admin/badges" },
            { label: "Seeding an instance", slug: "admin/seeding" },
          ],
        },
        {
          label: "Reference",
          items: [
            { label: "Configuration file", slug: "reference/configuration" },
            { label: "Command line", slug: "reference/cli" },
            { label: "Query language", slug: "reference/query-language" },
            { label: "HTTP API", slug: "reference/http-api" },
          ],
        },
        {
          label: "Project",
          items: [
            { label: "Architecture", slug: "about/architecture" },
            { label: "Compared with others", slug: "about/comparison" },
            { label: "Developing", slug: "develop" },
          ],
        },
      ],
    }),
    sitemap(),
  ],
});
