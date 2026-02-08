import { makeAutoObservable } from 'mobx'
import { commands, type FeeMode, type PrepareTxReqRes } from '../../bindings'
import { notifier } from '../../lib/notifier'

export class TransferStore {
	constructor() {
		makeAutoObservable(this)
	}

	address = ''
	setAddress(address: string) {
		this.address = address
	}
	isAddressValid = false
	setIsAddressValid(valid: boolean) {
		this.isAddressValid = valid
	}
	feeMode: FeeMode | null = 'Standard'
	setFeeMode(fm: FeeMode | null) {
		this.feeMode = fm
	}
	amount?: number
	setAmount(amount?: number) {
		this.amount = amount
	}
	selectedToken?: string
	setSelectedToken(token?: string) {
		this.selectedToken = token
	}
	preconfirmInfo?: PrepareTxReqRes
	setPreconfirmInfo(res?: PrepareTxReqRes) {
		this.preconfirmInfo = res
	}
	txHash?: string
	setTxHash(h?: string) {
		this.txHash = h
	}
	isEstimating = false
	setIsEstimating(v: boolean) {
		this.isEstimating = v
	}
	isSending = false
	setIsSending(v: boolean) {
		this.isSending = v
	}

	get disabled() {
		return (
			!this.address ||
			!this.isAddressValid ||
			!this.amount ||
			!this.selectedToken
		)
	}

	async verifyAddress() {
		const r = await commands.ethVerifyAddress(this.address)
		if (r.status === 'error') {
			this.setIsAddressValid(false)
			return
		}
		this.setIsAddressValid(true)
	}

	async createTrasaction() {
		if (!this.amount) throw Error('amount is not set')
		if (!this.selectedToken) throw Error('token symbol not set')
		this.setIsEstimating(true)
		const r = await commands.ethPrepareSendTx({
			amount: this.amount.toString(),
			fee_mode: this.feeMode ?? 'Standard',
			recipient: this.address,
			token_address: this.selectedToken,
		})
		this.setIsEstimating(false)
		if (r.status === 'error') {
			notifier.err(r.error)
			throw Error(r.error)
		}
		this.setPreconfirmInfo(r.data)
	}

	async signAndSend() {
		this.setIsSending(true)
		const r = await commands.ethSignAndSendTx()
		this.setIsSending(false)
		if (r.status === 'error') {
			notifier.err(r.error)
			this.setPreconfirmInfo(undefined)
			throw Error(r.error)
		}
		this.setTxHash(r.data)
	}
}
