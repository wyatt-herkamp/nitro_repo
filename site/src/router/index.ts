import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import InstallView from '@/views/admin/InstallView.vue'
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
    }
  ]
})

export default router
