<script lang="ts">
  import { root_store } from '../../../view_model/root.svelte'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { handle_err } from '../../../lib/handle_err'
  import AccountSelector from '../../../components/AccountSelector.svelte'
  import Copyable from '../../../components/Copyable.svelte'
  import NetworkStatus from '../../../components/NetworkStatus.svelte'
  import { TokenTracker } from '../view_model/token_tracker.svelte'
  import Balances from './Balances.svelte'
  import TrackToken from './TrackToken.svelte'
  import Transfer from './Transfer.svelte'

  const eth = $derived(root_store.wallet.eth)
  const tracker = new TokenTracker(() => eth.getBalance())
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
  <Balances
    {eth}
    anvil={!!root_store.settings.config?.eth.anvil}
    ontrack={() => tracker.open()}
  />
</section>

<Transfer {eth} />
<TrackToken vm={tracker} />

<style>
  .account-status {
    gap: 8px 24px;
  }
</style>
