import {defineConfig} from 'vitepress'

export default defineConfig({
    lang: 'en-US',
    title: 'Nitro_Repo',
    description: 'A Fast Artifact Manager',
    lastUpdated: true,
    themeConfig: {
        nav: [
            {text: 'Home', link: '/', activeMatch: '^/$'},
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
                link: 'https://github.com/wyatt-herkamp/nitro_repo/releases'
            }
        ],
        socialLinks: [
            {icon: 'github', link: 'https://github.com/wyatt-herkamp/nitro_repo'}
        ],
        sidebar: {
            '/': generalInfo(),
            '/sysAdmin/': sysAdminBar(),
            '/knowledge/': knowledgeBaseBar()
        },
    },

})

function generalInfo() {
    return [
        {
            text: 'Nitro Repo',
            items: [
                {text: 'What is Nitro Repo?', link: '/'},
                {text: 'Features', link: '/features'},
                {text: 'Compiling', link: '/compiling'},
                {text: 'Contributing', link: '/contributing'},
            ]
        }
    ]
}

function knowledgeBaseBar() {
    return [
        {
            text: 'User Management',
            items: [
                {text: 'User Permissions', link: '/knowledge/userpermissions'},
            ]
        }, {
            text: 'Repositories',
            items: [
                {text: 'Artifact Types', link: '/knowledge/ArtifactTypes'}
            ]
        }, {
            text: 'Other',
            items: [
                {text: 'Internal Workings', link: '/knowledge/InternalWorkings'}
            ]
        },

    ]
}

function sysAdminBar() {
    return [
        {
            text: 'Installing',
            items: [
                {text: 'Prepping your System', link: '/sysAdmin/'},

            ]
        }
    ]
}
