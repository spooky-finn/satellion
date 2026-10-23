<script lang="ts">
  import type { TransferVM } from '../view_model/transfer.vm.svelte'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { handle_err } from '../../../lib/handle_err'
  import Copyable from '../../../components/Copyable.svelte'

  let { t }: { t: TransferVM } = $props()
</script>

<h2 class="review-title">Transaction sent</h2>
{#if t.broadcast_result}
  <Copyable value={t.broadcast_result.tx_id} label="Copy transaction ID" />
  <button
    class="btn preset-tonal-primary"
    onclick={() =>
      openUrl(
        `https://mempool.space/tx/${t.broadcast_result!.tx_id}`,
      ).catch(handle_err)}
  >
    Open explorer
  </button>
{/if}
<button class="btn btn-plain" onclick={() => t.reset()}>Send another</button>

<style>
  .review-title {
    font-size: 16px;
    font-weight: 500;
    letter-spacing: 0;
  }
</style>
