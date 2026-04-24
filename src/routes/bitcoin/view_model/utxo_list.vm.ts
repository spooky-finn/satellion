import { makeAutoObservable } from 'mobx'
import type { UtxoDto } from '../../../bindings/btc'
import { Loader } from '../../../view_model/loader'

export class UtxoListVM {
  readonly loader = new Loader()
  constructor() {
    makeAutoObservable(this)
  }

  is_open = false
  selection_mode: boolean = false

  open(selection_mode?: boolean) {
    this.is_open = true
    this.selection_mode = selection_mode ?? false
  }

  close() {
    this.is_open = false
  }

  utxo: UtxoDto[] = []

  get total_value_sat() {
    return this.utxo.reduce((acc, utxo) => acc + BigInt(utxo.value), 0n)
  }

  _selected_utxo: number[] = []
  select_utxo(index: number) {
    if (this._selected_utxo.includes(index)) {
      this._selected_utxo = this._selected_utxo.filter(each => each !== index)
      return
    }

    this._selected_utxo.push(index)
  }

  get selected_utxo(): UtxoDto[] {
    return this._selected_utxo.map(i => this.utxo[i])
  }

  get selected_utxo_total_value() {
    return this.selected_utxo.reduce((acc, each) => {
      acc += Number(each.value)
      return acc
    }, 0)
  }

  reset() {
    this._selected_utxo = []
  }
}
