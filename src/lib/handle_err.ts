import type { Result } from '../bindings'
import { notifier } from './notifier'

/** Turn a generated command result into a value or a catchable error. */
export const unwrap_result = <T>(result: Result<T, string>): T => {
  if (result.status === 'error') throw new Error(result.error)

  return result.data
}

export const handle_err = (error: unknown) => {
  notifier.err(String(error))
}
