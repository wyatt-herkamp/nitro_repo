import { useValidationStore } from '@/stores/validations'
import type { PasswordRules } from '@/types/base'
export interface BaseValidationType {
  id: string
  message: string
  // If true, this validation will be ignored if any other validation has failed
  ignoreIfOthersFailed?: boolean
  isAsync: boolean
}
export type KeyPressAction =
  | string
  | {
      badKey: string
      replacedKey: string
    }
export interface AsyncValidationType extends BaseValidationType {
  validate: (value: string, originalValue?: string) => Promise<boolean>
  isAsync: true
}
export interface SyncValidationType extends BaseValidationType {
  validate: (value: string, originalValue?: string) => boolean
  isAsync: false
}
export type ValidationType = AsyncValidationType | SyncValidationType

export async function checkValidations(
  validations: ValidationType[],
  value: string
): Promise<{
  isValid: boolean
  validationResults: Record<string, boolean>
}> {
  const validationResults: Record<string, boolean> = {}
  // Current time
  const startTime = Date.now()
  let isCompletelyValid = true
  for (const validation of validations) {
    if (validation.ignoreIfOthersFailed && !isCompletelyValid) {
      console.debug(`Ignoring validation ${validation.id} because others have failed`)
      continue
    }
    let isValid
    if (validation.isAsync) {
      isValid = await validation.validate(value)
    } else {
      isValid = validation.validate(value)
    }
    if (isCompletelyValid && !isValid) {
      isCompletelyValid = false
    }
    console.debug(`Validation ${validation.id} is ${isValid}`)
    validationResults[validation.id] = isValid
  }
  // Time taken to validate
  const timeToValidate = Date.now() - startTime
  console.log(`Time taken to validate: ${timeToValidate}ms`)
  return {
    isValid: isCompletelyValid,
    validationResults: validationResults
  }
}

/**
 * Must be alphanumeric, underscores, and dashes
 */
export const VALID_NAME_TYPE_VALIDATION: ValidationType = {
  id: 'valid-name-type',
  message: 'Can only contain letters, numbers, underscores and dashes.',
  validate: isValidNameType,
  isAsync: false
}
export const USERNAME_VALIDATIONS: ValidationType[] = [
  VALID_NAME_TYPE_VALIDATION,
  {
    id: 'username-min-length',
    message: 'Must be at least 3 characters long.',
    validate: (value: string) => value.length >= 3,
    isAsync: false
  },
  {
    id: 'username-max-length',
    message: 'No Longer than 32 characters.',
    validate: (value: string) => value.length <= 32,
    isAsync: false
  },
  {
    id: 'username-availability',
    message: 'Username is available.',
    validate: isUsernameAvailable,
    isAsync: true
  }
]
export const EMAIL_VALIDATIONS: ValidationType[] = [
  // TODO: Add email validation
  {
    id: 'email-availability',
    message: 'Email is available.',
    validate: isEmailAvailable,
    isAsync: true
  }
]

/**
 *  Must be alphanumeric, underscores, and dashes
 * @param value Check if the value is a valid name type
 *
 */
export function isValidNameType(value: string): boolean {
  return /^[a-zA-Z0-9_-]*$/.test(value)
}

async function isUsernameAvailable(value: string, originalValue?: string): Promise<boolean> {
  if (originalValue) {
    if (value === originalValue) {
      return true
    }
  }
  const validationsStore = useValidationStore()
  const isAvailable = await validationsStore.isUsernameInUse(value)
  if (isAvailable === undefined) {
    console.error(`Unable to determine if ${value} is available`)
    return false
  }
  return isAvailable
}
async function isEmailAvailable(value: string, originalValue?: string) {
  if (originalValue) {
    if (value === originalValue) {
      return true
    }
  }
  const validationsStore = useValidationStore()
  const isAvailable = await validationsStore.isEmailInUse(value)
  if (isAvailable === undefined) {
    console.error(`Unable to determine if ${value} is available`)
    return false
  }
  return isAvailable
}

export const URL_SAFE_BAD_CHARS = [' ']

export function passwordValidationRules(
  actualPasswordRules: PasswordRules
): Array<SyncValidationType> {
  const validations: Array<SyncValidationType> = []

  if (actualPasswordRules.min_length !== 0) {
    validations.push({
      id: 'password-min-length',
      message: `Password must be at least ${actualPasswordRules.min_length} characters long`,
      validate: (value: string) => {
        return value.length >= actualPasswordRules.min_length
      },
      isAsync: false
    })
  }
  if (actualPasswordRules.require_uppercase) {
    validations.push({
      id: 'password-require-uppercase',
      message: 'Password must contain at least one uppercase letter',
      validate: (value: string) => {
        return /[A-Z]/.test(value)
      },
      isAsync: false
    })
  }
  if (actualPasswordRules.require_lowercase) {
    validations.push({
      id: 'password-require-lowercase',
      message: 'Password must contain at least one lowercase letter',
      isAsync: false,
      validate: (value: string) => {
        return /[a-z]/.test(value)
      }
    })
  }
  if (actualPasswordRules.require_number) {
    validations.push({
      id: 'password-require-number',
      message: 'Password must contain at least one number',
      isAsync: false,
      validate: (value: string) => {
        return /\d/.test(value)
      }
    })
  }
  if (actualPasswordRules.require_special) {
    validations.push({
      id: 'password-require-special',
      message: 'Password must contain at least one special character',
      isAsync: false,
      validate: (value: string) => {
        return /[!@#$%^&*(),.?":{}|<>]/.test(value)
      }
    })
  }
  return validations
}
