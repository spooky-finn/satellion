import { listen, type Event } from '@tauri-apps/api/event'
import { makeAutoObservable } from 'mobx'
import type { SyncProgress, SyncStatus } from '../bindings'

type SyncState = {
  status: 'completed' | 'in progress' | 'failed'
  height: number
}

export class BitcoinWallet {
  constructor() {
    makeAutoObservable(this)

    listen('btc_sync', (event: Event<SyncProgress>) => {
      this.setSync(event.payload.status, event.payload.height)
      console.log('btc_sync', event.payload)
    })
  }

  address!: string

  sync?: SyncState
  setSync(status: SyncStatus, height: number | null) {
    if (!this.sync) {
      this.sync = {
        status: 'in progress',
        height: 0
      }
    }

    this.sync.status = status
    if (height) {
      this.sync.height = height
    }
  }
}
