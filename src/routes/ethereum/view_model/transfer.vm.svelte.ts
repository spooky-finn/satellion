import {
  commands,
  type FeeMode,
  type TransferEstimation,
} from '../../../bindings/eth'
import { commands as shared_commands } from '../../../bindings'
import { AddressInputVM } from '../../../components/address_input.svelte'
import { unwrap_result } from '../../../lib/handle_err'

const validAmount = (amount: string | undefined) =>
  !!amount && /^\d*\.?\d+$/.test(amount) && /[1-9]/.test(amount)

/** Binds a fee estimate to the exact transfer inputs reviewed by the user. */
export class TransferVM {
  is_open = $state(false)
  readonly address = new AddressInputVM(address =>
    shared_commands.validateAddress('Ethereum', address),
  )
  fee_mode = $state<FeeMode>('Minimal')
  amount = $state<string | undefined>(undefined)
  token = $state<string | undefined>(undefined)
  estimation = $state.raw<TransferEstimation | undefined>(undefined)
  tx_hash = $state<string | undefined>(undefined)
  is_estimating = $state(false)
  sending = $state(false)
  error = $state<string | undefined>(undefined)
  private revision = 0
  private reviewed?: string

  set_open(open: boolean) {
    if (!this.sending && !this.is_estimating) this.is_open = open
  }

  set_fee_mode(mode: FeeMode) {
    this.fee_mode = mode
    this.invalidate()
  }

  set_amount(amount?: string) {
    this.amount = amount
    this.invalidate()
  }

  set_token(token?: string) {
    this.token = token
    this.invalidate()
  }

  set_tx_hash(hash?: string) {
    this.tx_hash = hash
  }

  get has_valid_amount() {
    return validAmount(this.amount)
  }

  get amount_error() {
    if (!this.amount || /^\d+\.$/.test(this.amount)) return undefined
    if (!/^\d*\.?\d*$/.test(this.amount))
      return 'Enter a decimal amount, for example 0.01.'
    if (!/[1-9]/.test(this.amount)) return 'Amount must be greater than zero.'

    return undefined
  }

  get disabled() {
    return !this.address.is_valid || !this.has_valid_amount || !this.token
  }

  /** Identify every field that can change the transaction or its fee estimate. */
  private signature() {
    return JSON.stringify([
      this.address.val,
      this.amount,
      this.token,
      this.fee_mode,
    ])
  }

  /** Reject pending estimates and require review again after an edit. */
  invalidate() {
    this.revision++
    this.estimation = undefined
    this.reviewed = undefined
    this.error = undefined
  }

  reset() {
    this.invalidate()
    this.address.reset()
    this.fee_mode = 'Minimal'
    this.amount = undefined
    this.token = undefined
    this.tx_hash = undefined
  }

  async estimate() {
    if (this.disabled || this.is_estimating || this.sending) return

    const revision = ++this.revision
    const signature = this.signature()

    this.estimation = undefined
    this.error = undefined
    this.is_estimating = true

    try {
      const estimation = await commands
        .estimateTransfer({
          amount: this.amount!,
          fee_mode: this.fee_mode,
          recipient: this.address.val,
          token_address: this.token!,
        })
        .then(unwrap_result)

      if (revision === this.revision && signature === this.signature()) {
        this.estimation = estimation
        this.reviewed = signature
      }
    } catch (error) {
      this.error = String(error)
    } finally {
      this.is_estimating = false
    }
  }

  async execute() {
    if (this.sending || !this.estimation || this.reviewed !== this.signature())
      return

    this.sending = true
    this.error = undefined

    try {
      this.tx_hash = await commands.executeTransfer().then(unwrap_result)
    } catch (error) {
      this.invalidate()
      this.error = String(error)
    } finally {
      this.sending = false
    }
  }
}
