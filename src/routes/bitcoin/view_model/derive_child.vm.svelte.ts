import { commands, type Proposal as ProposalDto } from '../../../bindings/btc'
import { unwrap_result } from '../../../lib/handle_err'
import { Loader } from '../../../view_model/loader.svelte'
import { Proposal } from '../proposal'

/** Derives a labeled receive address for the selected index and proposal. */
export class DeriveChildVM {
  readonly loader = new Loader()
  is_open = $state(false)

  set_is_open(o: boolean) {
    this.is_open = o
  }

  label = $state<string | undefined>(undefined)

  setLabel(l: string) {
    this.label = l
  }

  index = $state<number | null>(null)

  set_index(i: number | null) {
    this.index = i
  }

  address = $state<string | null>(null)
  proposal = $state<ProposalDto>(Proposal.Taproot)

  setProposal(proposal: ProposalDto) {
    this.proposal = proposal
  }

  async next_unused_key_index() {
    const index = await commands.nextUnusedIndex().then(unwrap_result)

    this.index = index
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
