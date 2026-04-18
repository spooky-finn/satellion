import { makeAutoObservable, runInAction } from 'mobx'
import { commands, type DerivedAddressDto } from '../../../bindings/btc'
import { unwrap_result } from '../../../lib/handle_err'
import { Loader } from '../../../view_model/loader'

export class ChildAddressListVM {
  readonly loader = new Loader()

  constructor() {
    makeAutoObservable(this)
  }

  is_open = false
  set_open(o: boolean) {
    this.is_open = o
  }

  addresses: DerivedAddressDto[] = []

  async fetch() {
    const addresses = await commands.getExternalAddresess().then(unwrap_result)
    runInAction(() => {
      this.addresses = addresses
    })
  }
}
