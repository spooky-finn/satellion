import { commands, type TrackedTokenInfo } from '../../../bindings/eth'
import { unwrap_result } from '../../../lib/handle_err'

/** Local token tracking dialog state, shared with the balances toolbar. */
export class TokenTracker {
  is_open = $state(false)
  tokenAddress = $state('')
  tracked = $state.raw<TrackedTokenInfo | undefined>(undefined)
  busy = $state(false)
  error = $state('')

  constructor(private readonly onTracked: () => Promise<void>) {}

  open() {
    this.tracked = undefined
    this.tokenAddress = ''
    this.error = ''
    this.is_open = true
  }

  close() {
    this.is_open = false
  }

  async track() {
    this.busy = true
    this.error = ''

    try {
      this.tracked = await commands
        .trackToken(this.tokenAddress.trim())
        .then(unwrap_result)
      await this.onTracked()
    } catch (cause) {
      this.error = String(cause)
    } finally {
      this.busy = false
    }
  }
}
