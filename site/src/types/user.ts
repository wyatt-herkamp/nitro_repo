import { RepositoryActions } from './base'

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
