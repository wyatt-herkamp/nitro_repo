import ProfileHome from '@/views/profile/ProfileHome.vue'
import ProfileSideNav from '@/components/nav/ProfileSideNav.vue'
import ProfileTokens from '@/views/profile/ProfileTokens.vue'
import ProfileLoginSettings from '@/views/profile/ProfileLoginSettings.vue'
import TokenCreate from './TokenCreate.vue'
const defaultProfileMeta = {
  sideBar: ProfileSideNav,
  requiresAuth: true
}
export const profileTokenTag = 'profileTokens'
export const profileRoutes = [
  {
    path: '/profile',
    name: 'profile',
    component: ProfileHome,
    meta: defaultProfileMeta
  },

  {
    path: '/profile/login',
    name: 'profileLogin',
    component: ProfileLoginSettings,
    meta: defaultProfileMeta
  },
  {
    path: '/profile/tokens',
    name: 'profileTokens',
    component: ProfileTokens,
    meta: {
      ...defaultProfileMeta,
      tag: profileTokenTag
    }
  },
  {
    path: '/profile/token/create',
    name: 'profileTokenCreate',
    component: TokenCreate,
    meta: {
      ...defaultProfileMeta,
      tag: profileTokenTag
    }
  }
]
