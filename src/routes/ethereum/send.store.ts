import { makeAutoObservable } from 'mobx'
import { commands, PrepareTxReqRes } from '../../bindings'
import { notifier } from '../../components/notifier'

export class EthereumSendStore {
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

  amount: number | null = null
  setAmount(amount: number) {
    this.amount = amount
  }

  selectedToken: string | null = null
  setSelectedToken(token: string | null) {
    this.selectedToken = token
  }

  preconfirmInfo: PrepareTxReqRes | null = null
  setPreconfirmInfo(res: PrepareTxReqRes | null) {
    this.preconfirmInfo = res
  }

  txHash!: string
  setTxHash(h: string) {
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

  async createTrasaction(walletId: number) {
    if (!this.amount) throw Error('amount is not set')
    if (!this.selectedToken) throw Error('token symbol not set')
    this.setIsEstimating(true)
    const r = await commands.ethPrepareSendTx(
      walletId,
      this.selectedToken,
      this.amount.toString(),
      this.address
    )
    this.setIsEstimating(false)
    if (r.status === 'error') {
      notifier.err(r.error)
      throw Error(r.error)
    }
    this.setPreconfirmInfo(r.data)
  }

  async signAndSend(walletId: number) {
    this.setIsSending(true)
    const r = await commands.ethSignAndSendTx(walletId)
    this.setIsSending(false)
    if (r.status === 'error') {
      notifier.err(r.error)
      this.setPreconfirmInfo(null)
      throw Error(r.error)
    }
    this.setTxHash(r.data)
  }
}
