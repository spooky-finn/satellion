import type { Proposal as ProposalDto } from '../../bindings/btc'

export enum Proposal {
  SegWit = 'segwit',
  Taproot = 'taproot',
}

export const proposals = [
  Proposal.SegWit,
  Proposal.Taproot,
] as const satisfies readonly ProposalDto[]
