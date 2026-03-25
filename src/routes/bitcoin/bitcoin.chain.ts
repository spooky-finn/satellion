import { type Event, listen } from '@tauri-apps/api/event'
import { makeAutoObservable } from 'mobx'
import {
  type BitcoinUnlockDto,
  type BlockChain,
  commands,
  type SyncHeightUpdateEvent,
  type SyncNewUtxoEvent,
  type SyncProgressEvent,
  type SyncWarningEvent,
} from '../../bindings'
import { AccountSelectorVM } from '../../components/account_selector'
import { notifier } from '../../lib/notifier'
import { sat2btc } from './utils/amount_formatters'

export class BitcoinChain {
  readonly chain: BlockChain = 'Bitcoin'
  readonly account_selector = new AccountSelectorVM(this.chain, async _ => {
    await this.load_account_info()
  })

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

  init(unlock: BitcoinUnlockDto) {
    this.account_selector.init(unlock.accounts, unlock.accounts[0].index)
    this.init_with_account_info(unlock.active_account)
  }

  init_with_account_info(info: BitcoinUnlockDto['active_account']) {
    this.address = info.address
    this.total_balance_sat = info.total_balance
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

  async load_account_info() {
    const res = await commands.btcAccountInfo()
    if (res.status == 'error') {
      notifier.err(res.error)
      throw res.error
    }

    this.init_with_account_info(res.data)
  }
}
