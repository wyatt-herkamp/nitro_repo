import { defineStore } from 'pinia'
import { type Ref, ref } from 'vue'
import type { PasswordRules, PublicUser, ScopeDescription, SiteInfo } from '@/types/base'
import http from '@/http'
export const siteStore = defineStore(
  'siteStore',
  () => {
    const siteInfo: Ref<SiteInfo | undefined> = ref(undefined)
    const userLookup: Ref<Map<number, PublicUser>> = ref(new Map())
    const scopes: Ref<Array<ScopeDescription>> = ref([])

    async function getScopes(): Promise<Array<ScopeDescription>> {
      if (scopes.value.length > 0) {
        return scopes.value
      }
      return await http
        .get<Array<ScopeDescription>>('/api/info/scopes')
        .then((response) => {
          scopes.value = response.data
          return response.data
        })
        .catch(() => {
          return []
        })
    }
    /// The Site Info is pulled almost immediately after the store is created So this should be available
    /// However, to make typescript happy, we will default to a password rules object
    function getPasswordRulesOrDefault(): PasswordRules {
      if (siteInfo.value === undefined) {
        return {
          min_length: 8,
          require_uppercase: true,
          require_lowercase: true,
          require_number: true,
          require_special: true
        }
      }
      return siteInfo.value.password_rules
    }
    async function getInfo(): Promise<SiteInfo | undefined> {
      return await http
        .get<SiteInfo>('/api/info')
        .then((response) => {
          siteInfo.value = response.data
          console.log('Site info: ', siteInfo.value)
          return response.data
        })
        .catch((err) => {
          console.error('Unable to get site info', err)
          return undefined
        })
    }
    async function getUser(id: number): Promise<PublicUser | undefined> {
      if (userLookup.value.has(id)) {
        return userLookup.value.get(id)
      }
      return await http
        .get<PublicUser>(`/api/user/get/${id}`)
        .then((response) => {
          userLookup.value.set(id, response.data)
          return response.data
        })
        .catch(() => {
          return undefined
        })
    }
    function isInstalled(): boolean {
      if (siteInfo.value === undefined) {
        return false
      }
      return siteInfo.value.is_installed
    }
    return { siteInfo, scopes, getPasswordRulesOrDefault, getScopes, getInfo, isInstalled, getUser }
  },
  {
    persist: false
  }
)
