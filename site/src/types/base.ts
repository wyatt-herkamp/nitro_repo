export interface SmallIdentification {
  name: string
}
export interface User {
  id: number
  name: string
  username: string
  email: string
  admin: boolean
  user_manager: boolean
  storage_manager: boolean
  repository_manager: boolean
  default_repository_actions: RepositoryActions[]
  created_at: Date
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
  password_rules: PasswordRules
}
export interface PasswordRules {
  min_length: number
  require_uppercase: boolean
  require_lowercase: boolean
  require_number: boolean
  require_special: boolean
}

export interface RepositoryActions {
  can_read: boolean
  can_write: boolean
  can_edit: boolean
}

export function formatDate(date: Date) {
  return `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`
}
