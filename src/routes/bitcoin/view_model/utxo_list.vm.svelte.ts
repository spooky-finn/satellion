import { commands, type UtxoView } from '../../../bindings/btc'
import { unwrap_result } from '../../../lib/handle_err'
import { Resource } from '../../../lib/resource.svelte'

const id = (utxo: UtxoView) => `${utxo.utxo_id.tx_id}:${utxo.utxo_id.vout}`

/** Keeps selected outpoints stable when refreshed outputs change order. */
export class UtxoListVM {
  readonly sync = new Resource(async () => {
    const revision = ++this.revision
    const utxos = await commands.syncUtxos().then(unwrap_result)

    if (revision !== this.revision) return

    this.utxo = utxos
    this.selected_ids = this.selected_ids.filter(key =>
      utxos.some(utxo => id(utxo) === key),
    )
  })
  private revision = 0
  is_open = $state(false)
  selection_mode = $state(false)
  utxo = $state.raw<UtxoView[]>([])
  selected_ids = $state<string[]>([])

  open(selection = false) {
    this.is_open = true
    this.selection_mode = selection
  }

  close() {
    this.is_open = false
  }

  get total_value_sat() {
    return this.utxo.reduce((total, utxo) => total + BigInt(utxo.value), 0n)
  }

  /** Group unconfirmed outputs by parent transaction for fee bump actions. */
  get pending_parent_txs() {
    const groups = new Map<string, bigint>()

    for (const utxo of this.utxo) {
      if (!utxo.confirmed)
        groups.set(
          utxo.utxo_id.tx_id,
          (groups.get(utxo.utxo_id.tx_id) ?? 0n) + BigInt(utxo.value),
        )
    }

    return Array.from(groups, ([parent_tx_id, value_sat]) => ({
      parent_tx_id,
      value_sat,
    }))
  }

  is_selected(utxo: UtxoView) {
    return this.selected_ids.includes(id(utxo))
  }

  select_utxo(index: number) {
    const utxo = this.utxo[index]

    if (!utxo) return

    const key = id(utxo)

    if (this.selected_ids.includes(key))
      this.selected_ids = this.selected_ids.filter(selected => selected !== key)
    else this.selected_ids.push(key)
  }

  get selected_utxo() {
    return this.utxo.filter(utxo => this.is_selected(utxo))
  }

  get selected_utxo_total_value() {
    return this.selected_utxo.reduce(
      (total, utxo) => total + BigInt(utxo.value),
      0n,
    )
  }

  reset() {
    this.selected_ids = []
  }
}
