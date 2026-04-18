import { makeAutoObservable, runInAction } from 'mobx'
import {
  commands,
  type FeeMode,
  type TransferEstimation,
} from '../../../bindings/eth'
import { commands as shared_commands } from '../../../bindings/index'
import { AddressInputVM } from '../../../components/address_input'
import { unwrap_result } from '../../../lib/handle_err'
import { notifier } from '../../../lib/notifier'

export class TransferVM {
  constructor() {
    makeAutoObservable(this)
  }

  is_open: boolean = false
  set_open(v: boolean) {
    this.is_open = v
  }

  readonly address = new AddressInputVM(addr =>
    shared_commands.validateAddress('Ethereum', addr),
  )

  fee_mode: FeeMode | null = 'Standard'
  set_fee_mode(fm: FeeMode | null) {
    this.fee_mode = fm
  }

  amount?: number
  set_amount(amount?: number) {
    this.amount = amount
  }

  token?: string
  set_token(token?: string) {
    this.token = token
  }

  estimation?: TransferEstimation

  tx_hash?: string
  set_tx_hash(h?: string) {
    this.tx_hash = h
  }

  is_estimating = false
  sending = false

  get disabled() {
    return (
      !this.address || !this.address.is_valid || !this.amount || !this.token
    )
  }

  async estimate() {
    if (!this.amount) throw Error('amount is not set')
    if (!this.token) throw Error('token symbol not set')

    this.is_estimating = true
    const estimation = await commands
      .estimateTransfer({
        amount: this.amount.toString(),
        fee_mode: this.fee_mode ?? 'Standard',
        recipient: this.address.val,
        token_address: this.token,
      })
      .then(res => unwrap_result(res))

    runInAction(() => {
      this.is_estimating = false
      this.estimation = estimation
    })
  }

  async execute() {
    this.sending = true
    const r = await commands.executeTransfer()

    runInAction(() => {
      this.sending = false
      if (r.status === 'error') {
        notifier.err(r.error)
        this.estimation = undefined
        throw Error(r.error)
      }
      this.set_tx_hash(r.data)
    })
  }
}
