import { makeAutoObservable } from 'mobx'
import { commands as shared_commands } from '../../bindings/index'
import { AddressInputVM } from '../components'

export class BitcoinTransferVM {
  constructor() {
    makeAutoObservable(this)
  }

  readonly address = new AddressInputVM(addr =>
    shared_commands.validateAddress('Bitcoin', addr),
  )
}
