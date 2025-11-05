import { makeAutoObservable } from 'mobx'
import { commands } from '../../bindings'

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

  async verifyAddress() {
    const r = await commands.ethVerifyAddress(this.address)
    if (r.status === 'error') {
      this.setIsAddressValid(false)
      return
    }
    this.setIsAddressValid(true)
  }
}
