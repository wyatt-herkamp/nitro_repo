export interface SmallIdentification {
  name: string
}
export interface User {
  id: number
  name: string
  username: string
  email: string
  permissions: UserPermissions
  created: Date
}
export interface PublicUser {
  id: number
  name: string
  username: string
  admin: boolean
}

export interface Session {
  user_id: number
  session_id: string
  expires: Date
  created: Date
}

export interface Me {
  user: User
  session: Session
}

export interface NewUser {
  username: string
  password: string
  email: string
  name: string
}
export interface SiteInfo {
  url?: string
  mode: string
  name: string
  description: string
  is_installed: boolean
  version: string
}
export interface UserPermissions {
  admin: boolean
  user_manager: boolean
  repository_manager: boolean
}
export function formatDate(date: Date) {
  return `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`
}
