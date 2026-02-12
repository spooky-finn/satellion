import { makeAutoObservable } from 'mobx'
import { commands, type UnlockMsg } from '../bindings'
import { notifier } from '../lib/notifier'
import { BitcoinChain } from '../routes/bitcoin/bitcoin.chain'
import { EthereumWallet } from '../routes/ethereum/wallet.store'

export class Wallet {
	readonly eth = new EthereumWallet()
	readonly btc = new BitcoinChain()

	name?: string
	constructor() {
		makeAutoObservable(this)
	}

	init(name: string, unlock: UnlockMsg) {
		this.name = name
		this.eth.init(unlock.ethereum)
		this.btc.init(unlock.bitcoin)
	}

	async forget(name: string) {
		const r = await commands.forgetWallet(name)
		if (r.status === 'error') {
			notifier.err(r.error)
			return
		}
	}

	async reset() {
		this.name = undefined
	}
}
