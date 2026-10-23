<script lang="ts">
  import Copyable from '../../../components/Copyable.svelte'
  import { handle_err } from '../../../lib/handle_err'
  import type { BitcoinWalletVM } from '../view_model/wallet.vm.svelte'

  let { btc }: { btc: BitcoinWalletVM } = $props()
</script>

{#if btc.address}
  <div class="soft-card stack compact">
    <div class="font-semibold">
      <Copyable value={btc.address} label="Copy address" />
    </div>
    <div class="row">
      <button
        class="btn preset-tonal-primary"
        onclick={(event) => {
            event.currentTarget.focus()
            btc.child_list.set_open(true)
          }}
      >
        Child addresses
      </button>
      <button
        class="btn preset-tonal-primary"
        onclick={(event) => {
            event.currentTarget.focus()
            btc.utxo_list.open()
          }}
      >
        Utxo
      </button>
      <button
        class="btn preset-filled-primary-500"
        disabled={btc.account_selector.account_loader.loading}
        onclick={(event) => {
            event.currentTarget.focus()
            btc.transfer.reset()
            btc.utxo_list.reset()
            btc.transfer.set_open(true)
          }}
      >
        Send
      </button>
      <button
        class="btn btn-plain ml-auto"
        disabled={btc.discovery_loader.loading}
        onclick={() => btc.discover_wallet().catch(handle_err)}
      >
        {btc.discovery_loader.loading ? 'Discovering…' : 'Discover'}
      </button>
    </div>
    {#if btc.discovery_loader.data}
      <p class="muted">
        Discovery found {btc.discovery_loader.data.accounts.length} accounts,
        added
        {btc.discovery_loader.data.paths_added}
        paths and
        {btc
            .discovery_loader.data.utxos_added}
        UTXOs
      </p>
    {/if}
  </div>
{/if}
