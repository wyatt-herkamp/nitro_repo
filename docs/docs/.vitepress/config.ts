import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'en-US',
  title: 'Nitro_Repo',
  description: 'A Fast Artifact Manager',
  lastUpdated: true,

  themeConfig: {
    repo: 'wherkamp/nitro_repo',
    docsDir: 'docs',
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
        text: 'Release Notes',
        link: 'https://github.com/wherkamp/wherkamp/releases'
      }
    ],

    sidebar: {
      '/sysAdmin/': sysAdminBar(),
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
      ]
    }
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
