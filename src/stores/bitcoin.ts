import { listen, type Event } from '@tauri-apps/api/event'
import { makeAutoObservable } from 'mobx'
import type { SyncProgress } from '../bindings'

type SyncStatus = {
  status: 'completed' | 'sync'
  height: number
}

export class BitcoinWallet {
  constructor() {
    makeAutoObservable(this)

    listen('btc_sync_progress', (event: Event<SyncProgress>) => {
      this.setSync({
        height: event.payload.height,
        status: 'sync'
      })
    })

    listen('btc_sync_completed', (event: Event<SyncProgress>) => {
      this.setSync({
        height: event.payload.height,
        status: 'completed'
      })
    })
  }

  address!: string

  sync?: SyncStatus
  setSync(s: SyncStatus) {
    this.sync = s
  }
}
