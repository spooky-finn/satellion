import { makeAutoObservable, runInAction } from 'mobx'
import type { UtxoView } from '../../../bindings/btc'
import { commands } from '../../../bindings/btc'
import { unwrap_result } from '../../../lib/handle_err'
import { Resource } from '../../../lib/resource'

export class UtxoListVM {
  readonly sync = new Resource(() => this._fetch())

  constructor() {
    makeAutoObservable(this)
  }

  private async _fetch(): Promise<void> {
    const utxo = await commands.syncUtxos().then(unwrap_result)
    runInAction(() => {
      this.utxo = utxo
    })
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

  utxo: UtxoView[] = []

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

  get selected_utxo(): UtxoView[] {
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
