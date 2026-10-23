<script lang="ts">
  import type { BitcoinWalletVM } from '../view_model/wallet.vm.svelte'
  import {
    TransferState,
    UtxoSelectionMethodKind,
  } from '../view_model/transfer.vm.svelte'
  import { display_sat } from '../utils/amount_formatters'
  import Modal from '../../../components/Modal.svelte'
  import AddressInput from '../../../components/AddressInput.svelte'
  import Copyable from '../../../components/Copyable.svelte'
  import DisplaySat from '../../../components/DisplaySat.svelte'
  import UtxoList from './UtxoList.svelte'
  import TransferResult from './TransferResult.svelte'
  import TransferReview from './TransferReview.svelte'

  let { btc }: { btc: BitcoinWalletVM } = $props()

  const t = $derived(btc.transfer)
  let selecting = $state(false)

  function selection(
    method: UtxoSelectionMethodKind,
    trigger: HTMLButtonElement,
  ) {
    trigger.focus()
    t.set_utxo_selection_method(method)

    if (method === UtxoSelectionMethodKind.Manual) selecting = true
  }

  async function send() {
    await t.execute()

    if (t.state === TransferState.Result) {
      btc.utxo_list.reset()
      await btc.load_account_info()
    }
  }
</script>

<Modal
  open={t.is_open}
  title="Send bitcoin"
  fullscreen
  busy={t.busy}
  onclose={() => t.set_open(false)}
>
  {#if t.state === TransferState.Result}
    <TransferResult {t} />
  {:else if t.state === TransferState.Sending}
    <TransferReview {btc} onsend={send} />
  {:else}
    <form
      class="stack"
      onsubmit={(event) => {
        event.preventDefault()
        t.estimate(btc.utxo_list.selected_utxo)
      }}
    >
      <AddressInput state={t.address} disabled={t.busy} />
      <div class="stack compact">
        <p class="text-sm">Utxo selection method</p>
        <div class="row">
          <div class="segmented" role="group" aria-label="Coin selection">
            <button
              type="button"
              class="btn"
              aria-pressed={t.utxo_selection_method ===
                UtxoSelectionMethodKind.Auto}
              disabled={t.busy}
              onclick={(event) =>
                selection(UtxoSelectionMethodKind.Auto, event.currentTarget)}
            >
              Auto
            </button>
            <button
              type="button"
              class="btn"
              aria-pressed={t.utxo_selection_method ===
                UtxoSelectionMethodKind.Manual}
              disabled={t.busy}
              onclick={(event) =>
                selection(UtxoSelectionMethodKind.Manual, event.currentTarget)}
            >
              Manual
            </button>
          </div>
          {#if t.show_utxo_select_button}
            <button
              type="button"
              class="btn btn-plain"
              onclick={(event) => {
                event.currentTarget.focus()
                selecting = true
              }}
            >
              Select inputs
            </button>
          {/if}
        </div>
        {#if t.show_utxo_select_button && btc.utxo_list.selected_utxo.length}
          <div class="soft-card outlined-card stack compact input-summary">
            <p>Inputs</p>
            {#each btc.utxo_list.selected_utxo as utxo (`${utxo.utxo_id.tx_id}:${utxo.utxo_id.vout}`)}
              <div class="row">
                <Copyable
                  value={`${utxo.utxo_id.tx_id}:${utxo.utxo_id.vout}`}
                  label="Copy selected output"
                  small
                /><span>{display_sat(utxo.value)}</span>
              </div>
            {/each}
            <hr>
            <DisplaySat
              value={btc.utxo_list.selected_utxo_total_value}
              price={btc.usd_price}
              label="In total"
            />
          </div>
        {/if}
      </div>
      <div class="row">
        <label class="input-unit"
          ><span class="sr-only">Amount in satoshis</span>
          <input
            class="input"
            placeholder="Amount"
            type="number"
            min="1"
            step="1"
            value={t.transfer_amount ?? ''}
            disabled={t.busy}
            oninput={(event) =>
              t.set_transfer_amount(
                Number.isNaN(event.currentTarget.valueAsNumber)
                  ? undefined
                  : event.currentTarget.valueAsNumber,
              )}
            required
          ><span>SAT</span></label
        >
        <span>{t.estimated_transfer_value(btc.usd_price)}</span>
      </div>
      <button
        class="btn preset-filled-primary-500"
        disabled={!t.valid ||
          t.busy ||
          (t.show_utxo_select_button && !btc.utxo_list.selected_utxo.length)}
      >
        {t.busy ? 'Estimating…' : 'Estimate'}
      </button>
    </form>
  {/if}
  {#if t.error}
    <p role="alert" class="error">{t.error}</p>
  {/if}
  <Modal
    open={t.is_open && selecting}
    title="Unspent transaction outputs"
    nested
    fullscreen
    onclose={() => {
      selecting = false
    }}
  >
    <UtxoList vm={btc.utxo_list} price={btc.usd_price} select />
    <button
      class="btn preset-filled-primary-500"
      onclick={() => {
        selecting = false
      }}
    >
      Use selected inputs
    </button>
  </Modal>
</Modal>

<style>
  .input-summary {
    width: fit-content;
    max-width: 100%;
    padding: 16px;
  }
</style>
