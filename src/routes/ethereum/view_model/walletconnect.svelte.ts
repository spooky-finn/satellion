import type { IWalletKit } from '@reown/walletkit'
import {
  commands,
  type WalletConnectReview,
  type WalletConnectSettings,
} from '../../../bindings/eth'
import { unwrap_result } from '../../../lib/handle_err'
import {
  build_proposal,
  type ConnectedApp,
  chain_number,
  type Proposal,
  type ProposalEvent,
  pairing_topic,
  type Request,
  type RequestEvent,
  type RpcError,
  request_payload,
  verified_peer,
} from '../walletconnect/protocol'
import { close_relay, disconnected, open_relay } from '../walletconnect/relay'

type PendingRequest = Request & {
  kind: 'request'
  expires_at: number
  review?: WalletConnectReview
}

type Pending = ({ kind: 'proposal' } & Proposal) | PendingRequest

interface Pairing {
  topic: string
  timer: ReturnType<typeof setTimeout>
}

const rejected = { code: 4001, message: 'User rejected the request' }
const expired = { code: -32000, message: 'The dApp request expired.' }
const connection_rejected = {
  code: 5000,
  message: 'User rejected the connection',
}

function same_request(event: RequestEvent, current?: RequestEvent) {
  return current?.id === event.id && current.topic === event.topic
}

export class WalletConnectVM {
  open = $state(false)
  uri = $state('')
  busy_approving = $state(false)
  error = $state('')
  result = $state('')
  address = $state('')
  settings = $state.raw<WalletConnectSettings>()
  sessions = $state.raw<ConnectedApp[]>([])

  private connecting = $state(false)
  private pairing = $state.raw<Pairing>()
  private pending = $state.raw<Pending>()
  private authorizing = $state.raw<Promise<void>>()

  private client?: IWalletKit
  private context = new AbortController()
  private cleanup = Promise.resolve()
  private request_timer?: ReturnType<typeof setTimeout>

  get busy() {
    return this.connecting || this.authorizing !== undefined
  }

  get waiting() {
    return this.pairing !== undefined
  }

  get proposal() {
    return this.pending?.kind === 'proposal' ? this.pending : undefined
  }

  get request() {
    return this.pending?.kind === 'request' ? this.pending : undefined
  }

  chain_name(chain: number) {
    return (
      this.settings?.chains.find(network => network.id === chain)?.name ??
      String(chain)
    )
  }

  async show(address: string) {
    this.open = true
    this.address = address
    this.error = ''
    const { signal } = this.context

    try {
      const settings = await commands
        .walletconnectSettings()
        .then(unwrap_result)
      if (!signal.aborted) this.settings = settings
    } catch {
      if (!signal.aborted) this.error = 'Could not load WalletConnect settings.'
    }
  }

  async connect() {
    const settings = this.settings
    if (
      !this.open ||
      !settings ||
      settings.disabled_reason ||
      this.busy ||
      this.waiting ||
      this.pending ||
      this.busy_approving
    )
      return

    const { signal } = this.context
    const uri = this.uri.trim()
    this.uri = ''
    this.error = ''
    this.result = ''
    this.connecting = true

    try {
      const topic = await pairing_topic(uri)
      if (signal.aborted) return
      const client =
        this.client ?? (await this.start(settings.project_id, signal))
      if (!client || signal.aborted || this.pending) return

      this.pairing = {
        topic,
        timer: setTimeout(() => {
          this.clear_pairing()
          this.error = 'Pairing timed out. Copy a new link from the dApp.'
        }, 45_000),
      }
      await client.pair({ uri })
    } catch {
      if (!signal.aborted) {
        this.clear_pairing()
        this.error =
          'Could not pair. Check the Project ID and connection, then try a fresh WalletConnect link.'
      }
    } finally {
      if (!signal.aborted) this.connecting = false
    }
  }

  private async start(project_id: string, signal: AbortSignal) {
    await this.cleanup
    if (signal.aborted) return
    const client = await open_relay(project_id, signal)
    if (!client) return
    if (signal.aborted) {
      await close_relay(client, false)
      return
    }

    this.attach_client(client)
    return client
  }

  private attach_client(client: IWalletKit) {
    this.client = client
    client.on('session_proposal', event => {
      if (client === this.client) void this.on_proposal(client, event)
    })
    client.on('session_request', event => {
      if (client === this.client) void this.on_request(client, event)
    })
    const remove_session = ({ topic }: { topic: string }) => {
      if (client === this.client) void this.remove_session(client, topic)
    }
    client.on('session_delete', remove_session)
    client.engine.signClient.events.on('session_expire', remove_session)
    client.on('proposal_expire', ({ id }) => {
      if (client === this.client && this.proposal?.id === id) {
        this.pending = undefined
        this.error = 'The connection proposal expired.'
      }
    })
    client.on('session_request_expire', ({ id }) => {
      const request = this.request
      if (request?.event.id === id)
        void this.expire_request(client, request.event, false)
    })
    client.on('session_authenticate', ({ id }) => {
      if (client !== this.client) return
      void client
        .rejectSessionAuthenticate({
          id,
          reason: {
            code: 5101,
            message: 'WalletConnect authentication is unsupported',
          },
        })
        .catch(() => {})
    })
  }

  private clear_pairing() {
    clearTimeout(this.pairing?.timer)
    this.pairing = undefined
  }

  private async on_proposal(client: IWalletKit, event: ProposalEvent) {
    if (this.proposal?.id === event.id) return
    const { settings, address, pairing } = this

    try {
      if (
        !settings ||
        !address ||
        !pairing ||
        event.params.pairingTopic !== pairing.topic ||
        this.pending ||
        this.busy_approving ||
        this.sessions.length >= 16
      ) {
        throw new Error('No matching WalletConnect pairing.')
      }
      const proposal = await build_proposal(event, settings, address)
      if (client !== this.client || this.proposal?.id === event.id) return
      if (this.pairing !== pairing || this.pending || this.busy_approving)
        throw new Error('No matching WalletConnect pairing.')

      this.clear_pairing()
      this.pending = { kind: 'proposal', ...proposal }
    } catch (error) {
      if (client !== this.client) return
      if (
        this.pairing === pairing &&
        event.params.pairingTopic === pairing?.topic
      )
        this.clear_pairing()
      this.error =
        error instanceof Error
          ? error.message
          : 'Could not approve the proposed networks and methods.'
      await client
        .rejectSession({
          id: event.id,
          reason: {
            code: 5100,
            message: 'Wallet cannot approve the proposed session',
          },
        })
        .catch(() => {})
    }
  }

  async decide_proposal(approved: boolean) {
    const { proposal, client } = this
    if (!proposal || !client || this.authorizing) return

    this.error = ''
    const operation = approved
      ? this.approve_session(client, proposal)
      : client.rejectSession({ id: proposal.id, reason: connection_rejected })
    this.authorizing = operation

    try {
      await operation
    } catch {
      if (client === this.client)
        this.error = 'Could not confirm the WalletConnect session.'
    } finally {
      if (client === this.client) {
        if (this.proposal === proposal) this.pending = undefined
        this.authorizing = undefined
      }
    }
  }

  private async approve_session(client: IWalletKit, proposal: Proposal) {
    const { topic } = await client.approveSession({
      id: proposal.id,
      namespaces: proposal.namespaces,
    })
    try {
      if (client !== this.client || !client.getActiveSessions()[topic])
        throw new Error('Wallet context changed.')
      await commands
        .walletconnectAuthorize({
          topic,
          address: proposal.address,
          chains: proposal.chains,
        })
        .then(unwrap_result)
      if (client !== this.client || !client.getActiveSessions()[topic])
        throw new Error('Wallet context changed.')
      this.sessions = [
        ...this.sessions,
        { ...proposal.peer, topic, chains: proposal.chains },
      ]
    } catch (error) {
      await this.remove_session(client, topic)
      await client
        .disconnectSession({ topic, reason: disconnected })
        .catch(() => {})
      throw error
    }
  }

  private async on_request(client: IWalletKit, event: RequestEvent) {
    await this.authorizing?.catch(() => {})
    if (client !== this.client) return
    if (same_request(event, this.request?.event)) return

    const app = this.sessions.find(session => session.topic === event.topic)
    const sdk_session = client.getActiveSessions()[event.topic]
    const chain = chain_number(event.params.chainId)
    const method = event.params.request.method
    if (
      !app ||
      !sdk_session ||
      !app.chains.some(
        grant => grant.id === chain && grant.methods.includes(method),
      )
    ) {
      await this.respond_error(client, event, {
        code: 4100,
        message: 'Request is outside the approved session',
      })
      return
    }

    if (this.pending || this.busy_approving) {
      await this.respond_error(client, event, {
        code: -32002,
        message: 'Another request is awaiting review',
      })
      return
    }

    let preparing: PendingRequest | undefined
    try {
      const peer = verified_peer(event.verifyContext, sdk_session.peer.metadata)
      if (app.verified && peer.verified && app.origin !== peer.origin)
        throw new Error('The verified dApp domain changed.')

      const request = { event, peer, chain }
      const payload = request_payload(request, sdk_session.expiry)
      preparing = {
        kind: 'request',
        ...request,
        expires_at: payload.expires_at,
      }
      this.error = ''
      this.result = ''
      this.pending = preparing
      this.open = true
      this.request_timer = setTimeout(
        () => void this.expire_request(client, event, true),
        payload.expires_at * 1000 - Date.now(),
      )
      const review = await commands
        .walletconnectPrepare(payload)
        .then(unwrap_result)

      if (client === this.client && this.pending === preparing)
        this.pending = { ...preparing, review }
      else if (!same_request(event, this.request?.event))
        await this.reject_native(client, event)
    } catch (error) {
      if (client !== this.client || (preparing && this.pending !== preparing))
        return
      if (preparing) this.clear_request()
      this.error =
        error instanceof Error
          ? error.message
          : 'The request could not be prepared.'
      await this.respond_error(client, event, {
        code: -32000,
        message: 'Wallet could not prepare the request',
      })
    }
  }

  private clear_request() {
    if (!this.request) return
    clearTimeout(this.request_timer)
    this.request_timer = undefined
    this.pending = undefined
  }

  private async expire_request(
    client: IWalletKit,
    event: RequestEvent,
    respond: boolean,
  ) {
    if (client !== this.client || this.request?.event !== event) return
    const approving = this.busy_approving
    this.clear_request()
    this.error = expired.message
    await this.reject_native(client, event)
    // An in-flight approval may still return a submitted transaction hash.
    if (respond && !approving) await this.respond_error(client, event, expired)
  }

  async decide_request(approved: boolean) {
    const { client, request } = this
    if (
      !client ||
      !request ||
      this.busy_approving ||
      (approved && !request.review)
    )
      return

    const { event, review } = request
    if (approved && request.expires_at * 1000 <= Date.now()) {
      await this.expire_request(client, event, true)
      return
    }
    this.error = ''
    this.busy_approving = true

    try {
      if (!approved) {
        this.clear_request()
        await this.reject_native(client, event)
        await this.respond_error(client, event, rejected)
        return
      }

      let result: string | null
      try {
        result = await commands
          .walletconnectApprove(event.topic, String(event.id))
          .then(unwrap_result)
      } catch (error) {
        const cancelled =
          error instanceof Error &&
          error.message === 'User rejected the wallet action'
        await this.respond_error(
          client,
          event,
          cancelled
            ? rejected
            : {
                code: -32000,
                message: 'Wallet could not complete the request',
              },
        )
        if (cancelled) return
        throw error
      }
      if (client !== this.client) return

      if (event.params.request.method === 'eth_sendTransaction')
        this.result = result ?? ''

      await client.respondSessionRequest({
        topic: event.topic,
        response: { id: event.id, jsonrpc: '2.0', result },
      })
      if (client !== this.client) return
      if (this.error === expired.message) this.error = ''

      const switch_chain = review?.switch_chain
      if (switch_chain) {
        await client.emitSessionEvent({
          topic: event.topic,
          chainId: `eip155:${switch_chain}`,
          event: {
            name: 'chainChanged',
            data: `0x${switch_chain.toString(16)}`,
          },
        })
      }
    } catch (error) {
      if (client === this.client) {
        this.error = this.result
          ? 'The transaction was sent, but WalletConnect could not deliver the response. Check the transaction before retrying.'
          : error instanceof Error
            ? error.message
            : 'WalletConnect could not complete the request.'
      }
    } finally {
      if (client === this.client) {
        if (this.request?.event === event) this.clear_request()
        this.busy_approving = false
      }
    }
  }

  private async respond_error(
    client: IWalletKit,
    event: RequestEvent,
    error: RpcError,
  ) {
    if (client !== this.client) return
    await client
      .respondSessionRequest({
        topic: event.topic,
        response: { id: event.id, jsonrpc: '2.0', error },
      })
      .catch(() => {
        if (client === this.client && !this.error)
          this.error = 'The relay could not deliver the response.'
      })
  }

  private async reject_native(client: IWalletKit, event: RequestEvent) {
    if (client !== this.client) return
    await commands
      .walletconnectReject(event.topic, String(event.id))
      .catch(() => {})
  }

  private async remove_session(client: IWalletKit, topic: string) {
    if (client === this.client) {
      this.sessions = this.sessions.filter(session => session.topic !== topic)
      if (this.request?.event.topic === topic) this.clear_request()
    }
    await commands.walletconnectDisconnect(topic).catch(() => {})
  }

  async disconnect(topic: string) {
    const client = this.client
    if (!client) return
    await this.remove_session(client, topic)
    try {
      await client.disconnectSession({ topic, reason: disconnected })
    } catch {
      if (client === this.client)
        this.error =
          'Disconnected locally. The relay could not notify the dApp.'
    }
  }

  close() {
    if (this.request) {
      void this.decide_request(false)
    } else if (this.proposal) {
      void this.decide_proposal(false)
    } else {
      this.clear_pairing()
      this.open = false
      this.uri = ''
    }
  }

  reset(notify_dapps = true) {
    const client = this.client
    this.context.abort()
    this.context = new AbortController()
    this.client = undefined
    this.authorizing = undefined
    this.clear_pairing()
    this.clear_request()
    this.open = false
    this.uri = ''
    this.error = ''
    this.result = ''
    this.connecting = false
    this.busy_approving = false
    this.address = ''
    this.settings = undefined
    this.pending = undefined
    this.sessions = []

    if (client) this.cleanup = close_relay(client, notify_dapps).catch(() => {})
  }
}
