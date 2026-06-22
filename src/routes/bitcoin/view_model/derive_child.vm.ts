import { makeAutoObservable, runInAction } from 'mobx'
import { commands, type Proposal as ProposalDto } from '../../../bindings/btc'
import { unwrap_result } from '../../../lib/handle_err'
import { Loader } from '../../../view_model/loader'
import { Proposal } from '../proposal'

export class DeriveChildVM {
  readonly loader = new Loader()
  constructor() {
    makeAutoObservable(this)
  }
  is_open = false
  set_is_open(o: boolean) {
    this.is_open = o
  }
  label?: string
  setLabel(l: string) {
    this.label = l
  }
  index: number | null = null
  set_index(i: number | null) {
    this.index = i
  }
  address: string | null = null
  proposal: ProposalDto = Proposal.Taproot
  setProposal(proposal: ProposalDto) {
    this.proposal = proposal
  }

  async next_unused_key_index() {
    const index = await commands.nextUnusedIndex().then(unwrap_result)
    runInAction(() => {
      this.index = index
    })
  }

  async derive() {
    if (!this.label) throw Error('label is not set')
    if (this.index === null) throw Error('index is not set')

    this.address = null
    this.loader.start()
    const address = await commands
      .deriveExternalAddress(this.label, this.index, this.proposal)
      .then(unwrap_result)
      .finally(() => this.loader.stop())

    this.address = address
  }
}
