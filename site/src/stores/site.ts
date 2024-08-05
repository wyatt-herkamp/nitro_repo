import { defineStore } from 'pinia'
import { type Ref, ref } from 'vue'
import type { PublicUser, SiteInfo } from '@/types/base'
import http from '@/http'
export const siteStore = defineStore(
  'siteStore',
  () => {
    const siteInfo: Ref<SiteInfo | undefined> = ref(undefined)
    const userLookup: Ref<Map<number, PublicUser>> = ref(new Map())

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
    return { siteInfo, getInfo, isInstalled, getUser }
  },
  {
    persist: false
  }
)
