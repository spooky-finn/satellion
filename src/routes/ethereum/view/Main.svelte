<script lang="ts">
  import { root_store } from '../../../view_model/root.svelte'
  import { commands, type TrackedTokenInfo } from '../../../bindings/eth'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { handle_err, unwrap_result } from '../../../lib/handle_err'
  import { notifier } from '../../../lib/notifier'
  import AccountSelector from '../../../components/AccountSelector.svelte'
  import Copyable from '../../../components/Copyable.svelte'
  import Modal from '../../../components/Modal.svelte'
  import Loading from '../../../components/Loading.svelte'
  import Transfer from './Transfer.svelte'
  import NetworkStatus from '../../../components/NetworkStatus.svelte'
  import Icon from '../../../components/Icon.svelte'

  const eth = $derived(root_store.wallet.eth)
  let tracking = $state(false)
  let tokenAddress = $state('')
  let tracked = $state.raw<TrackedTokenInfo | undefined>(undefined)
  let busy = $state(false)
  let error = $state('')

  function formatBalance(balance: string) {
    const [whole, fraction] = balance.split('.')

    return fraction ? `${whole}.${fraction.slice(0, 6)}` : whole
  }

  async function track() {
    busy = true
    error = ''

    try {
      tracked = await commands
        .trackToken(tokenAddress.trim())
        .then(unwrap_result)
      await eth.getBalance()
    } catch (cause) {
      error = String(cause)
    } finally {
      busy = false
    }
  }

  async function untrack(address: string) {
    await commands.untrackToken(address).then(unwrap_result)
    await eth.removeTokenFromBalance(address)
  }

  async function fund() {
    const message = await commands
      .anvilSetInitialBalances(eth.address)
      .then(unwrap_result)

    notifier.ok(message)
    await eth.getBalance()
  }

  async function refresh() {
    await Promise.all([eth.getBalance(), eth.getChainInfo()])
  }
</script>

<section class="stack compact" aria-label="Ethereum wallet">
  <h1 class="sr-only">Ethereum wallet</h1>
  <div class="row between account-status">
    <AccountSelector vm={eth.account_selector} />
    <NetworkStatus
      height={eth.chainInfo?.block_number}
      hash={eth.chainInfo?.block_hash}
      price={eth.usd_price}
    />
  </div>
  {#if eth.address}
    <div class="soft-card stack compact">
      <div class="font-semibold">
        <Copyable value={eth.address} label="Copy address" />
      </div>
      <div class="row">
        <button
          class="btn preset-tonal-primary"
          onclick={() =>
            openUrl(`https://etherscan.io/address/${eth.address}`).catch(
              handle_err,
            )}
        >
          Open explorer
        </button>
        <button
          class="btn preset-filled-primary-500"
          disabled={eth.account_selector.account_loader.loading}
          onclick={(event) => {
            event.currentTarget.focus()
            eth.transfer.reset()
            eth.transfer.set_open(true)
          }}
        >
          Send
        </button>
      </div>
    </div>
  {/if}
  <div class="soft-card row between items-start">
    <div class="token-list stack gap-1">
      {#if eth.balance.loading}
        <Loading label="Refreshing balances…" />
      {/if}
      {#if eth.balance_error}
        <p class="error" role="alert">{eth.balance_error}</p>
      {/if}
      {#each eth.balance.data?.tokens ?? [] as token (token.address)}
        <div class="token-row row">
          <span class="token-symbol">{token.symbol}</span>
          <span class="token-balance mono" title={token.balance}
            >{formatBalance(token.balance)}</span
          >
          {#if token.symbol !== 'ETH'}
            <button
              class="btn-icon"
              aria-label={`Untrack ${token.symbol}`}
              title="Do not track"
              onclick={() => untrack(token.address).catch(handle_err)}
            >
              <Icon name="minus" />
            </button>
          {/if}
        </div>
      {:else}
        {#if !eth.balance.loading && !eth.balance_error}
          <p>Tokens not found</p>
        {/if}
      {/each}
    </div>
    <div class="row token-actions">
      {#if root_store.settings.config?.eth.anvil}
        <button
          class="btn-icon"
          aria-label="Fund Anvil account"
          title="Set initial Anvil balances (10 ETH + 9,999,999 USDT)"
          onclick={() => fund().catch(handle_err)}
        >
          <Icon name="plus" />
        </button>
      {/if}
      <button
        class="btn btn-plain btn-neutral font-normal"
        onclick={(event) => {
          event.currentTarget.focus()
          tracked = undefined
          tokenAddress = ''
          error = ''
          tracking = true
        }}
      >
        Track token
      </button>
      <button
        class="btn-icon"
        aria-label="Refresh balances"
        title="Refresh balances"
        disabled={eth.balance.loading}
        onclick={() => refresh().catch(handle_err)}
      >
        <Icon name="refresh" size={24} />
      </button>
    </div>
  </div>
</section>
<Transfer {eth} />
<Modal
  open={tracking}
  title="Track a token"
  {busy}
  onclose={() => {
    tracking = false
  }}
>
  {#if tracked}
    <p role="status">{tracked.symbol} is now tracked.</p>
    <button
      class="btn preset-filled-primary-500"
      onclick={() => {
        tracking = false
      }}
    >
      Done
    </button>
  {:else}
    <form
      class="stack"
      onsubmit={(event) => {
        event.preventDefault()
        track()
      }}
    >
      <label class="field"
        ><span class="sr-only">Token contract address</span>
        <input
          class="input"
          placeholder="Token contract address"
          autocomplete="off"
          spellcheck="false"
          bind:value={tokenAddress}
          disabled={busy}
          required
        ></label
      >
      <button
        class="btn preset-filled-primary-500"
        disabled={busy || !tokenAddress.trim()}
      >
        {busy ? 'Tracking…' : 'Track token'}
      </button>
    </form>
  {/if}
  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</Modal>

<style>
  .account-status {
    gap: 8px 24px;
  }

  .token-list {
    flex: 1;
    padding: 2px 0;
  }

  .token-row {
    flex-wrap: nowrap;
    padding: 2px 0;
  }

  .token-symbol {
    min-width: 48px;
    font-weight: 500;
  }

  .token-balance {
    min-width: 0;
    padding-inline: 8px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .token-actions {
    margin-left: auto;
  }

  .token-actions .font-normal {
    font-weight: 400;
  }
</style>
