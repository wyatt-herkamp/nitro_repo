import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import InstallView from '@/views/admin/InstallView.vue'
import BrowseView from '@/views/BrowseView.vue'
import ProfileView from '@/views/ProfileView.vue'
import LoginView from '@/views/LoginView.vue'
import LogoutView from '@/views/LogoutView.vue'
import AdminHome from '@/views/admin/AdminHome.vue'
declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth?: boolean
  }
}
const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/admin/install',
      name: 'AdminInstall',
      component: InstallView
    },
    {
      path: '/browse',
      name: 'browse',
      component: BrowseView
    },
    {
      path: '/profile',
      name: 'profile',
      component: ProfileView
    },
    {
      path: '/login',
      name: 'login',
      component: LoginView
    },
    {
      path: '/logout',
      name: 'logout',
      component: LogoutView
    },
    {
      path: '/admin',
      name: 'admin',
      component: AdminHome
    }
  ]
})

export default router
