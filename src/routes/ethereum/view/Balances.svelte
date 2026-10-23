<script lang="ts">
  import type { EthereumWalletVM } from '../view_model/wallet.vm.svelte'
  import { commands } from '../../../bindings/eth'
  import { handle_err, unwrap_result } from '../../../lib/handle_err'
  import { notifier } from '../../../lib/notifier'
  import Loading from '../../../components/Loading.svelte'
  import Icon from '../../../components/Icon.svelte'

  let {
    eth,
    anvil,
    ontrack,
  }: { eth: EthereumWalletVM; anvil: boolean; ontrack: () => void } = $props()

  function formatBalance(balance: string) {
    const [whole, fraction] = balance.split('.')

    return fraction ? `${whole}.${fraction.slice(0, 6)}` : whole
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
    {#if anvil}
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
        ontrack()
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

<style>
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
