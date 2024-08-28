import AdminNav from '@/components/nav/AdminNav.vue'
import AdminHome from '@/views/admin/AdminHome.vue'
import InstallView from '@/views/admin/InstallView.vue'
import CreateRepositoryView from '@/views/admin/repository/CreateRepositoryView.vue'
import RepositoryListView from '@/views/admin/repository/RepositoryListView.vue'
import ViewRepositoryView from '@/views/admin/repository/ViewRepositoryView.vue'
import CreateStorageView from '@/views/admin/storage/CreateStorageView.vue'
import StorageListView from '@/views/admin/storage/StorageListView.vue'
import ViewStorage from '@/views/admin/storage/ViewStorage.vue'
import UserCreateView from '@/views/admin/user/UserCreateView.vue'
import UserListView from '@/views/admin/user/UserListView.vue'
import UserPage from '@/views/admin/user/UserPage.vue'
import path from 'path'
import AdminSystem from './AdminSystem.vue'
const defaultAdminMeta = {
  sideBar: AdminNav,
  requiresAuth: true
}
export const adminUserTag = 'admin-users'
export const adminRepositoryTag = 'admin-repositories'
export const adminStorageTag = 'admin-storages'
export const adminRoutes = [
  {
    path: '/admin/install',
    name: 'AdminInstall',
    component: InstallView
  },
  {
    path: '/admin',
    name: 'admin',
    component: AdminHome,
    meta: defaultAdminMeta
  },
  {
    path: '/admin/users',
    name: 'UsersList',
    component: UserListView,
    meta: {
      ...defaultAdminMeta,
      requiresUserManager: true,
      tag: adminUserTag
    }
  },
  {
    path: '/admin/user/create',
    name: 'UserCreate',
    component: UserCreateView,
    meta: {
      ...defaultAdminMeta,
      requiresUserManager: true,
      tag: adminUserTag
    }
  },
  {
    path: '/admin/user/:id',
    name: 'ViewUser',
    component: UserPage,
    meta: {
      ...defaultAdminMeta,
      requiresUserManager: true,
      tag: adminUserTag
    }
  },
  {
    path: '/admin/repositories',
    name: 'RepositoriesList',
    component: RepositoryListView,
    meta: {
      ...defaultAdminMeta,
      tag: adminRepositoryTag
    }
  },
  {
    path: '/admin/repository/:id',
    name: 'AdminViewRepository',
    component: ViewRepositoryView,
    meta: {
      ...defaultAdminMeta,
      tag: adminRepositoryTag
    }
  },
  {
    path: '/admin/repositories/create',
    name: 'RepositoryCreate',
    component: CreateRepositoryView,
    meta: {
      ...defaultAdminMeta,
      tag: adminRepositoryTag,
      requiresRepositoryManager: true
    }
  },

  {
    path: '/admin/storages',
    name: 'StorageList',
    component: StorageListView,
    meta: {
      ...defaultAdminMeta,
      tag: adminStorageTag,
      requiresRepositoryManager: true
    }
  },
  {
    path: '/admin/storage/create',
    name: 'StorageCreate',
    component: CreateStorageView,
    meta: {
      ...defaultAdminMeta,
      tag: adminStorageTag,
      requiresRepositoryManager: true
    }
  },
  {
    path: '/admin/storage/:id',
    name: 'ViewStorage',
    component: ViewStorage,
    meta: {
      ...defaultAdminMeta,
      tag: adminStorageTag,
      requiresRepositoryManager: true
    }
  },
  {
    path: '/admin/system',
    name: 'SystemSettings',
    component: AdminSystem,
    meta: defaultAdminMeta
  }
]
