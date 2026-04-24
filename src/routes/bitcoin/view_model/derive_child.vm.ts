import { makeAutoObservable, runInAction } from 'mobx'
import { commands } from '../../../bindings/btc'
import { unwrap_result } from '../../../lib/handle_err'
import { Loader } from '../../../view_model/loader'

export class DeriveChildVM {
  readonly loader = new Loader()
  constructor() {
    makeAutoObservable(this)
  }
  isOpen = false
  setIsOpen(o: boolean) {
    this.isOpen = o
  }
  label?: string
  setLabel(l: string) {
    this.label = l
  }
  index: number | null = null
  setIndex(i: number | null) {
    this.index = i
  }
  address: string | null = null

  async getAvaiableIndex() {
    const index = await commands
      .unoccupiedDeriviationIndex()
      .then(unwrap_result)
    runInAction(() => {
      this.index = index
    })
  }

  async derive() {
    if (!this.label) throw Error('label is not set')
    if (!this.index) throw Error('index is not set')

    this.address = null
    this.loader.start()
    const address = await commands
      .deriveExternalAddress(this.label, this.index)
      .then(unwrap_result)
      .finally(() => this.loader.stop())

    this.address = address
  }
}
