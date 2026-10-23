import {
  commands,
  type BroadcastTxResponse,
  type BuildTxResponse,
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
  Error,
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
  estimateion = $state.raw<BuildTxResponse | undefined>(undefined)
  busy = $state(false)

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
    if (!this.valid || this.busy) return

    if (this.show_utxo_select_button && !selected.length) {
      this.error = 'Select at least one input.'

      return
    }

    this.busy = true
    this.error = undefined

    try {
      this.estimateion = await commands
        .buildTx({
          value: String(this.transfer_amount),
          recipient: this.address.val,
          utxo_selection_method: this.show_utxo_select_button
            ? { Manual: selected.map(utxo => utxo.utxo_id) }
            : { Auto: this.transfer_amount! },
        })
        .then(unwrap_result)
      this.state = TransferState.Sending
    } catch (error) {
      this.error = String(error)
    } finally {
      this.busy = false
    }
  }

  /** Discard the reviewed transaction whenever its inputs change. */
  back_to_estimate() {
    this.estimateion = undefined
    this.error = undefined
    this.state = TransferState.Estimate
  }

  async execute() {
    if (this.busy || this.state !== TransferState.Sending || !this.estimateion)
      return

    this.busy = true
    this.error = undefined

    try {
      this.broadcast_result = await commands.broadcastTx({}).then(unwrap_result)
      this.state = TransferState.Result
    } catch (error) {
      this.error = String(error)
      this.back_to_estimate()
      this.error = String(error)
    } finally {
      this.busy = false
    }
  }

  reset() {
    this.address.reset()
    this.transfer_amount = undefined
    this.utxo_selection_method = UtxoSelectionMethodKind.Auto
    this.broadcast_result = undefined
    this.back_to_estimate()
  }
}
