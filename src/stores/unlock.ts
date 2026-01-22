import { makeAutoObservable, runInAction } from 'mobx'
import { commands } from '../bindings'
import { notifier } from '../components/notifier'
import { Loader } from './loader'
import type { Wallet } from './wallet'

export class Unlock {
	readonly loader = new Loader()
	constructor() {
		makeAutoObservable(this)
	}

	is_unlocked: boolean = false
	set_isunlocked(c: boolean) {
		this.is_unlocked = c
	}

	target_wallet: string | null = null
	set_target_wallet(w: string) {
		this.target_wallet = w
	}

	passphrase: string = ''
	set_passphrase(p: string) {
		this.passphrase = p
	}

	available_wallets: string[] = []
	set_available_wallets(w: string[]) {
		this.available_wallets = w
	}

	reset() {
		this.is_unlocked = false
		this.target_wallet = null
		this.passphrase = ''
		this.available_wallets = []
	}

	async load_available_wallets() {
		const r = await commands.listWallets()
		if (r.status === 'error') {
			notifier.err(r.error)
			throw Error(r.error)
		}

		runInAction(() => {
			this.available_wallets = r.data
			if (r.data.length === 1) {
				this.target_wallet = r.data[0]
			}
		})

		return r.data
	}

	async unlock_wallet(wallet_strore: Wallet) {
		if (!this.target_wallet) {
			throw new Error('No wallet selected to unlock')
		}
		this.loader.start()
		const r = await commands
			.unlockWallet(this.target_wallet, this.passphrase)
			.finally(() => this.loader.stop())

		if (r.status === 'error') {
			notifier.err(r.error)
			this.set_passphrase('')
			throw Error(r.error)
		}

		wallet_strore.init(this.target_wallet, r.data)
		runInAction(() => {
			this.is_unlocked = true
		})

		return r.data.last_used_chain
	}
}
