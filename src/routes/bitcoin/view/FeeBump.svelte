<script lang="ts">
  import { FeeBumpState } from '../view_model/fee_bump.vm.svelte'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { handle_err } from '../../../lib/handle_err'
  import Modal from '../../../components/Modal.svelte'
  import Copyable from '../../../components/Copyable.svelte'
  import DisplaySat from '../../../components/DisplaySat.svelte'
  import type { BitcoinWalletVM } from '../view_model/wallet.vm.svelte'

  let { btc }: { btc: BitcoinWalletVM } = $props()
</script>

<Modal
  open={!!btc.fee_bump.parent_tx_id}
  title="Bump fee (CPFP)"
  fullscreen
  busy={btc.fee_bump.state === FeeBumpState.Sending}
  onclose={() => btc.fee_bump.close()}
>
  {#if btc.fee_bump.result}
    <h2 class="result-title">Child transaction broadcast</h2>
    <div class="stack gap-1">
      <p class="text-sm">Child txid</p>
      <Copyable
        value={btc.fee_bump.result.child_tx_id}
        label="Copy child transaction ID"
        small
      />
    </div>
    <div class="stack gap-1">
      <p class="text-sm">Child fee</p>
      <DisplaySat
        value={btc.fee_bump.result.child_fee}
        price={btc.usd_price}
        label=""
        digits={2}
      />
    </div>
    <hr>
    <div class="row">
      <button
        class="btn preset-tonal-primary"
        onclick={() =>
          openUrl(
            `https://mempool.space/tx/${btc.fee_bump.result!.child_tx_id}`,
          ).catch(handle_err)}
      >
        Open explorer
      </button>
      <button class="btn btn-plain" onclick={() => btc.fee_bump.close()}>
        Close
      </button>
    </div>
  {:else}
    <div class="stack gap-1">
      <p class="text-sm">Parent transaction</p>
      <Copyable
        value={btc.fee_bump.parent_tx_id ?? ''}
        label="Copy parent transaction ID"
        small
      />
    </div>
    <p class="muted">
      Spends every output of the parent that belongs to this wallet into a new
      self-send. Miners with package-aware mempool policies will pull the parent
      in once the child pays enough.
    </p>
    <form
      class="stack"
      onsubmit={(event) => {
        event.preventDefault()
        btc.fee_bump.submit().catch(handle_err)
      }}
    >
      <label class="input-unit"
        ><span class="sr-only">Target fee rate (sat/vB)</span>
        <input
          class="input"
          type="number"
          placeholder="Fee rate"
          min="0.1"
          step="0.1"
          bind:value={btc.fee_bump.fee_rate_sat_vb}
          required
        ><span>sat/vB</span></label
      >
      <div class="row">
        <button
          type="button"
          class="btn btn-plain"
          disabled={btc.fee_bump.state === FeeBumpState.Sending}
          onclick={() => btc.fee_bump.close()}
        >
          Cancel
        </button>
        <button
          class="btn preset-filled-primary-500"
          disabled={btc.fee_bump.state === FeeBumpState.Sending}
        >
          {btc.fee_bump.state === FeeBumpState.Sending
            ? 'Broadcasting…'
            : 'Broadcast CPFP'}
        </button>
      </div>
    </form>
  {/if}
  {#if btc.fee_bump.error}
    <p class="error" role="alert">{btc.fee_bump.error}</p>
  {/if}
</Modal>

<style>
  .result-title {
    font-size: 16px;
    line-height: 20px;
    letter-spacing: 0;
    font-weight: 400;
  }
</style>
