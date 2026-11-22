import {
  commands,
  type BroadcastTxResponse,
  type BuildTxResponse,
  type TxRecord,
  type UtxoView,
} from '../../../bindings/btc'
import { commands as shared_commands } from '../../../bindings'
import { AddressInputVM } from '../../../components/address_input.svelte'
import { unwrap_result } from '../../../lib/handle_err'

export enum UtxoSelectionMethodKind {
  Auto = 'auto',
  Manual = 'manual',
}

export enum TransferState {
  Estimate,
  Sending,
  Result,
}

/** Builds a Bitcoin transaction for review before allowing it to be broadcast. */
export class TransferVM {
  readonly address = new AddressInputVM(address =>
    shared_commands.validateAddress('Bitcoin', address),
  )
  is_open = $state(false)
  utxo_selection_method = $state(UtxoSelectionMethodKind.Auto)
  transfer_amount = $state<number | undefined>(undefined)
  state = $state(TransferState.Estimate)
  broadcast_result = $state.raw<BroadcastTxResponse | undefined>(undefined)
  error = $state<string | undefined>(undefined)
  draft = $state.raw<BuildTxResponse | undefined>(undefined)
  busy = $state(false)
  recovery = $state.raw<
    | { status: 'idle' | 'loading' }
    | { status: 'failed'; error: string }
    | { status: 'ready'; attempts: TxRecord[] }
  >({ status: 'idle' })
  private revision = 0

  open() {
    this.reset()
    this.is_open = true
    return this.load_unresolved()
  }

  async load_unresolved() {
    if (this.recovery.status === 'loading') return
    const revision = this.revision
    this.recovery = { status: 'loading' }
    try {
      const attempts = await commands
        .listUnresolvedBtcBroadcasts()
        .then(unwrap_result)
      if (revision === this.revision)
        this.recovery = { status: 'ready', attempts }
    } catch (error) {
      if (revision === this.revision)
        this.recovery = { status: 'failed', error: String(error) }
    }
  }

  async retry(txid: string) {
    if (this.busy || this.recovery.status !== 'ready') return false
    return this.submit(() =>
      commands.retryBtcBroadcast(txid).then(unwrap_result),
    )
  }

  set_open(open: boolean) {
    if (!this.busy) this.is_open = open
  }

  set_utxo_selection_method(method: UtxoSelectionMethodKind) {
    this.utxo_selection_method = method
    this.back_to_estimate()
  }

  set_transfer_amount(amount?: number) {
    this.transfer_amount = amount
    this.back_to_estimate()
  }

  get show_utxo_select_button() {
    return this.utxo_selection_method === UtxoSelectionMethodKind.Manual
  }

  get valid() {
    return (
      this.address.is_valid &&
      Number.isSafeInteger(this.transfer_amount) &&
      (this.transfer_amount ?? 0) > 0
    )
  }

  estimated_transfer_value(price: number) {
    return this.transfer_amount
      ? `~ $${((price * this.transfer_amount) / 1e8).toFixed(2)}`
      : ''
  }

  async estimate(selected: UtxoView[]) {
    if (!this.valid || this.busy || this.recovery.status !== 'ready') return

    if (this.show_utxo_select_button && !selected.length) {
      this.error = 'Select at least one input.'

      return
    }

    const revision = this.revision
    this.busy = true
    this.error = undefined

    try {
      const draft = await commands
        .buildTx({
          value: String(this.transfer_amount),
          recipient: this.address.val,
          utxo_selection_method: this.show_utxo_select_button
            ? { Manual: selected.map(utxo => utxo.utxo_id) }
            : { Auto: this.transfer_amount! },
        })
        .then(unwrap_result)
      if (revision !== this.revision) return
      this.draft = draft
      this.state = TransferState.Sending
    } catch (error) {
      if (revision === this.revision) this.error = String(error)
    } finally {
      if (revision === this.revision) this.busy = false
    }
  }

  /** Discard the reviewed transaction whenever its inputs change. */
  back_to_estimate() {
    this.draft = undefined
    this.error = undefined
    this.state = TransferState.Estimate
  }

  async execute() {
    const draft = this.draft
    if (this.busy || this.state !== TransferState.Sending || !draft)
      return false
    return this.submit(() =>
      commands
        .broadcastTx({ draft_tx_id: draft.draft_tx_id })
        .then(unwrap_result),
    )
  }

  private async submit(send: () => Promise<BroadcastTxResponse>) {
    const revision = this.revision
    this.busy = true
    this.error = undefined

    try {
      const result = await send()
      if (revision !== this.revision) return false
      this.broadcast_result = result
      this.state = TransferState.Result
      return true
    } catch (error) {
      if (revision === this.revision) {
        this.back_to_estimate()
        this.error = String(error)
        await this.load_unresolved()
      }
      return false
    } finally {
      if (revision === this.revision) this.busy = false
    }
  }

  reset() {
    this.address.reset()
    this.transfer_amount = undefined
    this.utxo_selection_method = UtxoSelectionMethodKind.Auto
    this.broadcast_result = undefined
    this.revision++
    this.recovery = { status: 'idle' }
    this.is_open = false
    this.busy = false
    this.back_to_estimate()
  }
}
