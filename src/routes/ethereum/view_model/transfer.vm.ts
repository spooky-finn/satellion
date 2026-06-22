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

const is_valid_amount = (amount: string | undefined) =>
  Boolean(amount && /^\d*\.?\d+$/.test(amount) && /[1-9]/.test(amount))

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

  fee_mode: FeeMode = 'Minimal'
  set_fee_mode(fm: FeeMode) {
    this.fee_mode = fm
  }

  amount?: string
  set_amount(amount?: string) {
    this.amount = amount
  }

  get has_valid_amount() {
    return is_valid_amount(this.amount)
  }

  get amount_error() {
    if (!this.amount || /^\d+\.$/.test(this.amount)) return undefined
    if (!/^\d*\.?\d*$/.test(this.amount)) {
      return 'Enter a decimal amount, for example 0.01.'
    }
    if (!/[1-9]/.test(this.amount)) return 'Amount must be greater than zero.'
    return undefined
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

  reset() {
    this.address.reset()
    this.fee_mode = 'Minimal'
    this.amount = undefined
    this.token = undefined
    this.estimation = undefined
    this.tx_hash = undefined
  }

  is_estimating = false
  sending = false

  get disabled() {
    return !this.address.is_valid || !this.has_valid_amount || !this.token
  }

  async estimate() {
    const amount = this.amount
    if (!amount || !is_valid_amount(amount)) throw Error('amount is not valid')
    if (!this.token) throw Error('token symbol not set')

    this.is_estimating = true

    const estimation = await commands
      .estimateTransfer({
        amount,
        fee_mode: this.fee_mode,
        recipient: this.address.val,
        token_address: this.token,
      })
      .then(res => unwrap_result(res))
      .finally(() => {
        runInAction(() => {
          this.is_estimating = false
        })
      })

    runInAction(() => {
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
