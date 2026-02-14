import { type Event, listen } from '@tauri-apps/api/event'
import { makeAutoObservable } from 'mobx'
import type {
  SyncHeightUpdateEvent,
  SyncNewUtxoEvent,
  SyncProgressEvent,
  SyncWarningEvent,
  UnlockMsg,
} from '../../bindings'
import { notifier } from '../../lib/notifier'
import { sat2btc } from './utils/amount_formatters'

export class BitcoinChain {
  constructor() {
    makeAutoObservable(this)

    listen('btc_sync', (event: Event<SyncHeightUpdateEvent>) => {
      this.setHeight(event.payload.height)
      this.setStatus(event.payload.status)
    })
    listen('btc_sync_progress', (event: Event<SyncProgressEvent>) => {
      this.setProgress(event.payload.progress)
    })
    listen('btc_sync_warning', (event: Event<SyncWarningEvent>) => {
      this.setWarning(event.payload.msg)
    })
    listen('btc_sync_new_utxo', (event: Event<SyncNewUtxoEvent>) => {
      this.set_total_balance_sat(event.payload.total)
      notifier.ok(`Found new utxo ${sat2btc(event.payload.value)}`)
    })
  }

  init(unlock: UnlockMsg['bitcoin']) {
    this.address = unlock.address
    this.total_balance_sat = unlock.total_balance
  }

  address!: string
  usd_price = 0

  warning?: string
  setWarning(w?: string) {
    this.warning = w
  }

  height?: number
  setHeight(h: number) {
    this.height = h
    this.warning = undefined
  }
  progress: number = 0
  setProgress(p: number) {
    this.progress = p
    this.warning = undefined
  }
  status?: SyncHeightUpdateEvent['status']
  setStatus(s: SyncHeightUpdateEvent['status']) {
    this.status = s
    this.warning = undefined
  }

  total_balance_sat: string = '0'
  set_total_balance_sat(s: string) {
    this.total_balance_sat = s
  }
}
