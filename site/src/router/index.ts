import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import InstallView from '@/views/admin/InstallView.vue'
import BrowseView from '@/views/BrowseView.vue'
import ProfileView from '@/views/ProfileView.vue'
import LoginView from '@/views/LoginView.vue'
import LogoutView from '@/views/LogoutView.vue'
import AdminHome from '@/views/admin/AdminHome.vue'
import UserListView from '@/views/admin/user/UserListView.vue'
import UserCreateView from '@/views/admin/user/UserCreateView.vue'
import CreateStorageView from '@/views/admin/storage/CreateStorageView.vue'
import ViewStorage from '@/views/admin/storage/ViewStorage.vue'
import StorageListView from '@/views/admin/storage/StorageListView.vue'
import CreateRepositoryView from '@/views/admin/repository/CreateRepositoryView.vue'
import ViewRepositoryView from '@/views/admin/repository/ViewRepositoryView.vue'
import RepositoryListView from '@/views/admin/repository/RepositoryListView.vue'
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
    },
    {
      path: '/admin/users',
      name: 'UsersList',
      component: UserListView
    },
    {
      path: '/admin/user/create',
      name: 'UserCreate',
      component: UserCreateView
    },
    {
      path: '/admin/repositories',
      name: 'RepositoriesList',
      component: RepositoryListView
    },
    {
      path: '/admin/repositories/create',
      name: 'RepositoryCreate',
      component: CreateRepositoryView
    },
    {
      path: '/admin/repository/:id',
      name: 'ViewRepository',
      component: ViewRepositoryView
    },
    {
      path: '/admin/storages',
      name: 'StorageList',
      component: StorageListView
    },
    {
      path: '/admin/storage/create',
      name: 'StorageCreate',
      component: CreateStorageView
    },
    {
      path: '/admin/storage/:id',
      name: 'ViewStorage',
      component: ViewStorage
    }
  ]
})

export default router
