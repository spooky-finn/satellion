<script lang="ts">
  import Copyable from '../../../components/Copyable.svelte'
  import DisplaySat from '../../../components/DisplaySat.svelte'
  import type { BitcoinWalletVM } from '../view_model/wallet.vm.svelte'

  let { btc }: { btc: BitcoinWalletVM } = $props()
</script>

{#if btc.utxo_list.pending_parent_txs.length}
  <div class="soft-card warning-card stack compact">
    <h2 class="pending-title">Pending transactions</h2>
    <p class="muted">
      Outputs from these txs are still in the mempool. Bump the fee with a CPFP
      child to help them confirm.
    </p>
    {#each btc.utxo_list.pending_parent_txs as tx (tx.parent_tx_id)}
      <div class="row between">
        <div class="stack gap-0">
          <Copyable
            value={tx.parent_tx_id}
            label="Copy pending transaction ID"
            small
          />
          <DisplaySat
            value={tx.value_sat}
            price={btc.usd_price}
            label="Pending"
            digits={2}
          />
        </div>
        <button
          class="btn preset-filled-primary-500"
          onclick={(event) => {
              event.currentTarget.focus()
              btc.fee_bump.open(tx.parent_tx_id)
            }}
        >
          Bump fee
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .pending-title {
    font-size: 14px;
    line-height: 20px;
    letter-spacing: 0;
    font-weight: 500;
  }
</style>
