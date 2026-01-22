import { type Event, listen } from '@tauri-apps/api/event'
import { makeAutoObservable } from 'mobx'
import type {
	SyncHeightUpdateEvent,
	SyncProgressEvent,
	SyncWarningEvent,
} from '../../bindings'

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
	}

	address!: string
	usd_price!: string

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
}
