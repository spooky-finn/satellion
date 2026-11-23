import type { IWalletKit, WalletKitTypes } from '@reown/walletkit'
import type {
  WalletConnectChainGrant,
  WalletConnectRequest,
  WalletConnectSettings,
} from '../../../bindings/eth'

export type ProposalEvent = WalletKitTypes.SessionProposal
export type RequestEvent = WalletKitTypes.SessionRequest
type Namespaces = Parameters<IWalletKit['approveSession']>[0]['namespaces']

interface Peer {
  name: string
  origin: string
  verified: boolean
}

export interface ConnectedApp extends Peer {
  topic: string
  chains: WalletConnectChainGrant[]
}

export interface Proposal {
  id: number
  peer: Peer
  address: string
  namespaces: Namespaces
  chains: WalletConnectChainGrant[]
}

export interface Request {
  event: RequestEvent
  peer: Peer
  chain: number
}

export interface RpcError {
  code: number
  message: string
}

const now = () => Math.floor(Date.now() / 1000)

export async function pairing_topic(uri: string): Promise<string> {
  const { parseUri } = await import('@walletconnect/utils')
  const parsed = parseUri(uri)
  if (
    !uri.startsWith('wc:') ||
    parsed.version !== 2 ||
    !/^[a-f0-9]{64}$/i.test(parsed.topic) ||
    !/^[a-f0-9]{64}$/i.test(parsed.symKey) ||
    parsed.relay.protocol !== 'irn' ||
    (parsed.expiryTimestamp !== undefined && parsed.expiryTimestamp <= now())
  ) {
    throw new Error('Copy a fresh WalletConnect v2 link from the dApp QR code.')
  }
  return parsed.topic
}

export function chain_number(id: string): number {
  return /^eip155:[1-9]\d*$/.test(id) ? Number(id.slice(7)) : NaN
}

function dapp_origin(url: string): string {
  try {
    const origin = new URL(url)
    return origin.protocol === 'https:' || origin.protocol === 'http:'
      ? origin.origin
      : ''
  } catch {
    return ''
  }
}

export function verified_peer(
  context: ProposalEvent['verifyContext'],
  metadata: { name: string; url: string },
): Peer {
  const verification = context?.verified
  if (verification?.isScam || verification?.validation === 'INVALID') {
    throw new Error('WalletConnect flagged this dApp domain.')
  }

  const verified_origin = dapp_origin(verification?.origin ?? '')
  return {
    name: metadata.name.slice(0, 100),
    origin: verified_origin || dapp_origin(metadata.url) || 'Unknown domain',
    verified: verification?.validation === 'VALID' && !!verified_origin,
  }
}

export async function build_proposal(
  event: ProposalEvent,
  settings: WalletConnectSettings,
  address: string,
): Promise<Proposal> {
  const peer = verified_peer(
    event.verifyContext,
    event.params.proposer.metadata,
  )
  const { buildApprovedNamespaces } = await import('@walletconnect/utils')
  const chains = settings.chains.map(({ id }) => `eip155:${id}`)
  const namespaces = buildApprovedNamespaces({
    proposal: event.params,
    supportedNamespaces: {
      eip155: {
        chains,
        methods: settings.methods,
        events: ['accountsChanged', 'chainChanged'],
        accounts: chains.map(chain => `${chain}:${address}`),
      },
    },
  })

  const methods_by_chain = new Map<number, Set<string>>()
  for (const namespace of Object.values(namespaces)) {
    for (const account of namespace.accounts) {
      const id = chain_number(account.slice(0, account.lastIndexOf(':')))
      const methods = methods_by_chain.get(id) ?? new Set<string>()
      for (const method of namespace.methods) methods.add(method)
      methods_by_chain.set(id, methods)
    }
  }
  if (!methods_by_chain.size)
    throw new Error('The dApp did not request a supported EVM network.')

  return {
    id: event.id,
    peer,
    address,
    namespaces,
    chains: [...methods_by_chain].map(([id, methods]) => ({
      id,
      methods: [...methods],
    })),
  }
}

export function request_payload(
  { event, chain }: Request,
  session_expiry: number,
): WalletConnectRequest {
  const request = event.params.request
  const received_at = now()
  const expires_at = Math.min(
    request.expiryTimestamp ?? received_at + 300,
    session_expiry,
  )
  if (
    !Number.isInteger(expires_at) ||
    expires_at <= received_at ||
    expires_at > received_at + 7 * 24 * 60 * 60
  )
    throw new Error('The dApp request has an invalid or expired deadline.')

  return {
    topic: event.topic,
    id: String(event.id),
    chain,
    method: request.method,
    params_json: JSON.stringify(request.params ?? []),
    expires_at,
  }
}
