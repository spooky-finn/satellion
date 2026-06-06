import { makeAutoObservable, runInAction } from 'mobx'
import { type BumpFeeResponse, commands } from '../../../bindings/btc'
import { unwrap_result } from '../../../lib/handle_err'

export enum FeeBumpState {
  Idle,
  Sending,
  Result,
  Error,
}

export class FeeBumpVM {
  constructor(private readonly on_success: () => Promise<void> | void) {
    makeAutoObservable(this)
  }

  parent_tx_id?: string
  fee_rate_sat_vb: number = 5
  state = FeeBumpState.Idle
  result?: BumpFeeResponse
  error?: string

  open(parent_tx_id: string) {
    this.parent_tx_id = parent_tx_id
    this.state = FeeBumpState.Idle
    this.result = undefined
    this.error = undefined
  }

  close() {
    this.parent_tx_id = undefined
    this.state = FeeBumpState.Idle
    this.result = undefined
    this.error = undefined
  }

  set_fee_rate(v: number) {
    this.fee_rate_sat_vb = v
  }

  async submit() {
    if (!this.parent_tx_id) return
    if (!this.fee_rate_sat_vb || this.fee_rate_sat_vb <= 0) {
      this.error = 'fee rate must be greater than zero'
      return
    }
    this.state = FeeBumpState.Sending
    this.error = undefined

    await commands
      .bumpFeeCpfp({
        parent_tx_id: this.parent_tx_id,
        target_fee_rate_sat_vb: this.fee_rate_sat_vb,
      })
      .then(unwrap_result)
      .then(async r => {
        runInAction(() => {
          this.result = r
          this.state = FeeBumpState.Result
        })
        await this.on_success()
      })
      .catch(e => {
        runInAction(() => {
          this.error = String(e)
          this.state = FeeBumpState.Error
        })
      })
  }
}
