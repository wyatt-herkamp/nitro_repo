import type { RepositoryActions, RawUserPermissions } from './base'

export class UserPermissions {
  admin: boolean
  user_manager: boolean
  storage_manager: boolean
  repository_manager: boolean
  default_repository_permissions: RepositoryActions
  repository_permissions: Record<string, RepositoryActions>

  constructor(data: RawUserPermissions) {
    this.admin = data.admin
    this.user_manager = data.user_manager
    this.storage_manager = data.storage_manager
    this.repository_manager = data.repository_manager
    this.default_repository_permissions = data.default_repository_permissions
    this.repository_permissions = data.repository_permissions
  }

  equalsRawType(other: RawUserPermissions): boolean {
    return (
      this.admin === other.admin &&
      this.user_manager === other.user_manager &&
      this.storage_manager === other.storage_manager &&
      this.repository_manager === other.repository_manager &&
      compareRepositoryActions(
        this.default_repository_permissions,
        other.default_repository_permissions
      ) &&
      compareRepositoryPermissions(this.repository_permissions, other.repository_permissions)
    )
  }

  removeRepository(repository: string) {
    delete this.repository_permissions[repository]
  }

  getRepositoryActions(repository: string): RepositoryActions | undefined {
    return this.repository_permissions[repository]
  }

  addOrUpdateRepository(repository: string, actions: RepositoryActions) {
    this.repository_permissions[repository] = actions
  }

  updateUserPermissionsRequest(): any {
    return {
      admin: this.admin,
      user_manager: this.user_manager,
      storage_manager: this.storage_manager,
      repository_manager: this.repository_manager,
      default_repository_permissions: this.default_repository_permissions
    }
  }
}
export function compareRepositoryActions(a: RepositoryActions, b: RepositoryActions): boolean {
  return a.can_edit === b.can_edit && a.can_read === b.can_read && a.can_write === b.can_write
}
export function compareRepositoryPermissions(
  a: Record<string, RepositoryActions>,
  b: Record<string, RepositoryActions>
) {
  if (Object.keys(a).length !== Object.keys(b).length) return false
  for (const key in a) {
    if (!compareRepositoryActions(a[key], b[key])) return false
  }
  return true
}
