<script lang="ts">
  import type { BitcoinWalletVM } from '../view_model/wallet.vm.svelte'
  import DisplaySat from '../../../components/DisplaySat.svelte'
  import HoldToSend from '../../../components/HoldToSend.svelte'

  let { btc, onsend }: { btc: BitcoinWalletVM; onsend: () => Promise<void> } =
    $props()

  const t = $derived(btc.transfer)
</script>

<h2 class="review-title">Verify transfer</h2>
<div class="stack gap-1">
  <p class="text-sm">Recipient</p>
  <p class="mono">{t.address.val}</p>
</div>
<div class="stack gap-1">
  <p class="text-sm">Amount</p>
  <DisplaySat
    value={t.transfer_amount ?? 0}
    price={btc.usd_price}
    label=""
    digits={2}
  />
</div>
<div class="stack gap-1">
  <p class="text-sm">Network fee</p>
  <DisplaySat
    value={t.estimateion?.fee ?? 0}
    price={btc.usd_price}
    label=""
    digits={2}
  />
</div>
<hr>
<div class="stack gap-1">
  <p class="text-sm">Total</p>
  <DisplaySat
    value={(t.transfer_amount ?? 0) + (t.estimateion?.fee ?? 0)}
    price={btc.usd_price}
    label=""
    digits={2}
  />
</div>
<div class="row">
  <button
    class="btn btn-plain"
    disabled={t.busy}
    onclick={() => t.back_to_estimate()}
  >
    Back
  </button>
  <HoldToSend busy={t.busy} {onsend} />
</div>

<style>
  .review-title {
    font-size: 16px;
    font-weight: 500;
    letter-spacing: 0;
  }
</style>
