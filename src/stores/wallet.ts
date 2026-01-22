import { makeAutoObservable } from 'mobx'
import { commands, type UnlockMsg } from '../bindings'
import { notifier } from '../components/notifier'
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
		this.eth.address = unlock.ethereum.address
		this.eth.usd_price = Number(unlock.ethereum.usd_price).toFixed(0)

		this.btc.address = unlock.bitcoin.address
		this.btc.usd_price = Number(unlock.bitcoin.usd_price).toFixed(0)

		this.eth.getChainInfo()
		this.eth.getBalance()
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
