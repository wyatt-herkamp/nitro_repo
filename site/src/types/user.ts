import http from '@/http'
import { RepositoryActions, type UserResponseType } from './base'

export class User {
  id: number
  name: string
  username: string
  email: string
  admin: boolean
  user_manager: boolean
  system_manager: boolean
  default_repository_actions: Array<RepositoryActions>
  created_at: Date
  fullPermissions: FullPermissions | undefined
  constructor(user: UserResponseType) {
    this.id = user.id
    this.name = user.name
    this.username = user.username
    this.email = user.email
    this.admin = user.admin
    this.user_manager = user.user_manager
    this.system_manager = user.system_manager
    this.default_repository_actions = user.default_repository_actions
    this.created_at = new Date(user.created_at)
  }
  isAdminOrUserManager() {
    return this.admin || this.user_manager
  }
  isAdminSystemManager() {
    return this.admin || this.system_manager
  }

  async getAllPermissions(fromAdmin: boolean): Promise<FullPermissions | undefined> {
    if (!this.fullPermissions) {
      if (fromAdmin) {
        await http
          .get<FullPermissions>(`/api/user-management/get/${this.id}/permissions`)
          .then((response) => {
            this.fullPermissions = response.data
          })
      } else {
        await http.get<FullPermissions>(`/api/me/permissions`).then((response) => {
          this.fullPermissions = response.data
        })
      }
    }
    return this.fullPermissions
  }
  async updateMyOwnPassword(oldPassword: string, newPassword: string): Promise<void> {
    return await http.post('/api/change-password', {
      old_password: oldPassword,
      new_password: newPassword
    })
  }
  async updateOtherUserPassword(newPassword: string): Promise<void> {
    return await http.put(`/api/user-management/update/${this.id}/password`, {
      new_password: newPassword
    })
  }

  async refresh(fromAdmin: boolean): Promise<void> {
    if (fromAdmin) {
      const response = await http.get<UserResponseType>(`/api/user-management/get/${this.id}`)
      if (response.status === 200) {
        this.update(response.data)
      }
    } else {
      const response = await http.get<UserResponseType>('/api/me')
      if (response.status === 200) {
        this.update(response.data)
      }
    }
  }
  update(user: UserResponseType): void {
    this.id = user.id
    this.name = user.name
    this.username = user.username
    this.email = user.email
    this.admin = user.admin
    this.user_manager = user.user_manager
    this.system_manager = user.system_manager
    this.default_repository_actions = user.default_repository_actions
    this.created_at = new Date(user.created_at)
  }
}

export function hasRepositoryAction(
  actions: Array<RepositoryActions>,
  action: RepositoryActions
): boolean {
  return actions.includes(action)
}

export class RepositoryActionsType {
  can_read: boolean
  can_write: boolean
  can_edit: boolean
  constructor(actions: Array<RepositoryActions>) {
    this.can_read = actions.includes(RepositoryActions.Read)
    this.can_write = actions.includes(RepositoryActions.Write)
    this.can_edit = actions.includes(RepositoryActions.Edit)
  }
  update(actions: RepositoryActionsType): void {
    this.can_read = actions.can_read
    this.can_write = actions.can_write
    this.can_edit = actions.can_edit
  }
  updateFromArray(actions: Array<RepositoryActions>): void {
    this.can_read = actions.includes(RepositoryActions.Read)
    this.can_write = actions.includes(RepositoryActions.Write)
    this.can_edit = actions.includes(RepositoryActions.Edit)
  }
  asArray(): Array<RepositoryActions> {
    const actions: Array<RepositoryActions> = []

    if (this.can_read) {
      actions.push(RepositoryActions.Read)
    }

    if (this.can_write) {
      actions.push(RepositoryActions.Write)
    }

    if (this.can_edit) {
      actions.push(RepositoryActions.Edit)
    }

    return actions
  }
  equalsArray(actions: Array<RepositoryActions>): boolean {
    return (
      this.can_read === actions.includes(RepositoryActions.Read) &&
      this.can_write === actions.includes(RepositoryActions.Write) &&
      this.can_edit === actions.includes(RepositoryActions.Edit)
    )
  }
}

export interface FullPermissions {
  admin: boolean
  user_manager: boolean
  storage_manager: boolean
  repository_manager: boolean
  default_repository_actions: Array<RepositoryActions>
  repository_permissions: Record<string, Array<RepositoryActions>>
}
