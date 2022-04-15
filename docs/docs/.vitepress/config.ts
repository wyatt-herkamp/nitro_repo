import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'en-US',
  title: 'Nitro_Repo',
  description: 'A Fast Artifact Manager',
  lastUpdated: true,

  themeConfig: {
    repo: 'wherkamp/nitro_repo',
    docsDir: 'docs/docs',
    docsBranch: 'master',
    editLinks: true,
    editLinkText: 'Edit this page on GitHub',
    lastUpdated: 'Last Updated',

    nav: [
      { text: 'Home', link: '/', activeMatch: '^/$' },
      {
        text: 'System Admin',
        link: '/sysAdmin/',
        activeMatch: '^/sysAdmin/'
      },
      {
        text: 'Knowledge Base',
        link: '/knowledge/',
        activeMatch: '^/knowledge/'
      },
      {
        text: 'Release Notes',
        link: 'https://github.com/wherkamp/wherkamp/releases'
      }
    ],

    sidebar: {
      '/sysAdmin/': sysAdminBar(),
      '/knowledge/': knowledgeBaseBar(),
      '/': generalInfo()

    }
  }
})
function generalInfo() {
  return [
    {
      text: 'Nitro Repo',
      children: [
        { text: 'What is Nitro Repo?', link: '/' },
        { text: 'Features', link: '/features' },
        { text: 'Compiling', link: '/compiling' },
        { text: 'Contributing', link: '/contributing' },
      ]
    }
  ]
}
function knowledgeBaseBar() {
  return [
    {
      text: 'User Management',
      children: [
        { text: 'User Permissions', link: '/knowledge/userpermissions' },
      ]
    }, {
      text: 'Repositories',
      children: [
        { text: 'Artifact Types', link: '/knowledge/ArtifactTypes' }
      ]
    }, {
      text: 'Other',
      children: [
        { text: 'Internal Workings', link: '/knowledge/InternalWorkings' }
      ]
    },

  ]
}

function sysAdminBar() {
  return [
    {
      text: 'Installing',
      children: [
        { text: 'Prepping your System', link: '/sysAdmin/' },

      ]
    }
  ]
}
