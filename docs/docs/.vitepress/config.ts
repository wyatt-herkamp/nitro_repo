import { defineConfig } from "vitepress";

export default defineConfig({
  lang: "en-US",
  title: "Nitro_Repo",
  description: "A Fast Artifact Manager",
  lastUpdated: true,
  themeConfig: {
    nav: [
      { text: "Home", link: "/", activeMatch: "^/$" },
      {
        text: "System Admin",
        link: "/sysAdmin/",
        activeMatch: "^/sysAdmin/",
      },
      {
        text: "Knowledge Base",
        link: "/knowledge/",
        activeMatch: "^/knowledge/",
      },
      {
        text: "Repository Types",
        link: "/repositoryTypes/",
        activeMatch: "^/repositoryTypes/",
      },
      {
        text: "Release Notes",
        link: "https://github.com/wyatt-herkamp/nitro_repo/releases",
      },
    ],
    socialLinks: [
      { icon: "github", link: "https://github.com/wyatt-herkamp/nitro_repo" },
    ],
    sidebar: {
      "/": generalInfo(),
      "/sysAdmin/": sysAdminBar(),
      "/knowledge/": knowledgeBaseBar(),
      "/repositoryTypes/": repositoryTypesBar(),
    },
  },
});

function generalInfo() {
  return [
    {
      text: "Nitro Repo",
      items: [
        { text: "What is Nitro Repo?", link: "/" },
        { text: "Features", link: "/features" },
        { text: "Contributing", link: "/contributing" },
      ],
    },
  ];
}

function knowledgeBaseBar() {
  return [
    {
      text: "Other",
      items: [
        { text: "Internal Workings", link: "/knowledge/InternalWorkings" },
      ],
    },
  ];
}

function sysAdminBar() {
  return [
    {
      text: "Installing",
      items: [{ text: "Prepping your System", link: "/sysAdmin/" }],
    },
  ];
}

function repositoryTypesBar() {
  return [
    {
      text: "Maven",
      link: "/repositoryTypes/maven",
      items: [
        {
          text: "Maven Standard",
          link: "/repositoryTypes/maven/standard",
        },
        {
          text: "Nitro Deploy",
          link: "/repositoryTypes/maven/nitroDeploy",
        },
        {
          text: "Configs",
          link: "/repositoryTypes/maven/configs",
        },
      ],
    },
    {
      text: "NPM",
      link: "/repositoryTypes/npm",
      items: [
        {
          text: "NPM Standard",
          link: "/repositoryTypes/npm/standard",
        },
        {
          text: "Configs",
          link: "/repositoryTypes/npm/configs",
        },
        {
          text: "Common Issues",
          link: "/repositoryTypes/npm/errors",
        },
      ],
    },
  ];
}
