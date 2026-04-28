import { makeAutoObservable } from 'mobx'
import {
  type BroadcastResult,
  commands,
  type UtxoDto,
  type UtxoSelectionMethod,
} from '../../../bindings/btc'
import { commands as shared_commands } from '../../../bindings/index'
import { AddressInputVM } from '../../../components/address_input'
import { unwrap_result } from '../../../lib/handle_err'

export enum UtxoSelectionMethodName {
  Auto = 'auto',
  Manual = 'manual',
}

export enum TransferState {
  Estimate,
  Sending,
  Result,
  Error,
}

export class TransferVM {
  readonly address = new AddressInputVM(addr =>
    shared_commands.validateAddress('Bitcoin', addr),
  )
  constructor() {
    makeAutoObservable(this)
  }

  is_open = false
  set_open(o: boolean) {
    this.is_open = o
  }

  utxo_selection_method?: UtxoSelectionMethodName | null
  set_utxo_selection_method(v: UtxoSelectionMethodName | null) {
    this.utxo_selection_method = v
  }

  get show_utxo_select_button() {
    return this.utxo_selection_method === UtxoSelectionMethodName.Manual
  }

  transfer_amount?: number
  set_transfer_amount(v?: number) {
    this.transfer_amount = v
  }

  state = TransferState.Estimate
  broadcast_result?: BroadcastResult
  error?: string

  estimated_transfer_value(btc_price: number): string {
    if (!this.transfer_amount) return ''
    const estimated_value = (btc_price / 10 ** 8) * this.transfer_amount
    return `~ $${estimated_value.toFixed(2)}`
  }

  async estimate(selected_utxos: UtxoDto[]) {
    if (!this.transfer_amount) throw Error('transfer amount not set')

    const utxo_selection_method: UtxoSelectionMethod =
      this.utxo_selection_method === UtxoSelectionMethodName.Auto
        ? {
            Automatic: this.transfer_amount,
          }
        : { Manual: selected_utxos.map(each => each.utxo_id) }

    await commands
      .buildTx({
        value: this.transfer_amount.toString(),
        recipient: this.address.val,
        utxo_selection_method,
      })
      .then(unwrap_result)
      .then(() => {
        this.state = TransferState.Sending
      })
      .catch(e => {
        this.error = e
      })
  }

  async execute() {
    await commands
      .broadcastTx({})
      .then(unwrap_result)
      .then(r => {
        this.state = TransferState.Result
        this.broadcast_result = r
      })
      .catch(e => {
        this.state = TransferState.Estimate
        this.error = e
      })
  }

  reset() {
    this.address.reset()
    this.transfer_amount = undefined
    this.state = TransferState.Estimate
    this.broadcast_result = undefined
  }
}
