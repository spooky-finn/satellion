import { commands, type DerivedAddress } from '../../../bindings/btc'
import { unwrap_result } from '../../../lib/handle_err'
import { Loader } from '../../../view_model/loader.svelte'

/** External receive addresses shown in the child address dialog. */
export class ChildAddressListVM {
  readonly loader = new Loader()
  is_open = $state(false)

  set_open(o: boolean) {
    this.is_open = o
  }

  addresses = $state<DerivedAddress[]>([])

  async fetch() {
    const addresses = await commands.getExternalAddresess().then(unwrap_result)

    this.addresses = addresses
  }
}
