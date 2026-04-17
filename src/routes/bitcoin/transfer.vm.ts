import { makeAutoObservable } from 'mobx'
import { commands as shared_commands } from '../../bindings/index'
import { AddressInputVM } from '../components'

export enum UtxoSelectionMethodName {
  Auto = 'auto',
  Manual = 'manual',
}

export class BitcoinTransferVM {
  constructor() {
    makeAutoObservable(this)
  }

  is_open = false
  set_open(o: boolean) {
    this.is_open = o
  }

  readonly address = new AddressInputVM(addr =>
    shared_commands.validateAddress('Bitcoin', addr),
  )

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

  estimated_transfer_value(btc_price: number): string {
    if (!this.transfer_amount) return ''
    const estimated_value = (btc_price / 10 ** 8) * this.transfer_amount
    return `~ $${estimated_value.toFixed(2)}`
  }
}
