import type { Result } from '../bindings'
import { notifier } from './notifier'

type RpcResult<T> = Result<T, string>

export const unwrap_result = async <T>(result: RpcResult<T>) => {
  if (result.status === 'error') {
    notifier.err(result.error)
    throw result.error
  }
  return result.data
}

export const handle_err = async (e: any) => {
  notifier.err(e)
}
