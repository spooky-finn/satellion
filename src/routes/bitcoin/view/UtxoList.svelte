<script lang="ts">
  import type { UtxoListVM } from '../view_model/utxo_list.vm.svelte'
  import { display_sat, sat2usd } from '../utils/amount_formatters'
  import { handle_err } from '../../../lib/handle_err'
  import Loading from '../../../components/Loading.svelte'
  import Copyable from '../../../components/Copyable.svelte'
  import DisplaySat from '../../../components/DisplaySat.svelte'

  let {
    vm,
    price,
    select = false,
    disabled = false,
  }: {
    vm: UtxoListVM
    price: number
    select?: boolean
    disabled?: boolean
  } = $props()
</script>

<div class="stack compact">
  <div class="row between">
    <p>In total {vm.utxo.length} utxo</p>
    <button
      class="btn btn-plain"
      disabled={vm.sync.loading || disabled}
      onclick={() => vm.sync.refresh().catch(handle_err)}
    >
      Refresh outputs
    </button>
  </div>
  <DisplaySat value={vm.total_value_sat} {price} />
  {#if vm.sync.loading}
    <Loading label="Syncing outputs…" />
  {/if}
  {#if vm.sync.error}
    <p class="error" role="alert">{vm.sync.error}</p>
  {/if}
  {#if !vm.utxo.length && !vm.sync.loading}
    <p>No utxos yet.</p>
  {:else}
    <div class="table-wrap">
      <table class="table">
        <thead>
          <tr>
            {#if select}
              <th class="selection-column">
                <span class="sr-only">Select</span>
              </th>
            {/if}
            <th>Derivation path</th>
            <th>Label</th>
            <th>Transaction ID</th>
            <th>Amount</th>
            <th>Value</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {#each vm.utxo as utxo, index (`${utxo.utxo_id.tx_id}:${utxo.utxo_id.vout}`)}
            <tr class={vm.is_selected(utxo) ? 'selected-output' : ''}>
              {#if select}
                <td>
                  <input
                    class="checkbox"
                    type="checkbox"
                    aria-label={`Select output ${utxo.utxo_id.tx_id}:${utxo.utxo_id.vout}`}
                    checked={vm.is_selected(utxo)}
                    {disabled}
                    onchange={() => vm.select_utxo(index)}
                  >
                </td>
              {/if}
              <td class="mono text-xs whitespace-nowrap">{utxo.deriv_path}</td>
              <td class="whitespace-nowrap">{utxo.address_label ?? ''}</td>
              <td>
                <Copyable
                  value={utxo.utxo_id.tx_id}
                  label={`Copy transaction ${utxo.utxo_id.tx_id}`}
                />
              </td>
              <td class="mono whitespace-nowrap">{display_sat(utxo.value)}</td>
              <td class="mono whitespace-nowrap">
                {sat2usd(utxo.value, price, 2)}
              </td>
              <td>
                <span
                  class={[
                    'badge',
                    utxo.confirmed
                      ? 'preset-tonal-success'
                      : 'preset-tonal-warning',
                  ]}
                  >{utxo.confirmed ? 'Confirmed' : 'Pending'}</span
                >
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
  {#if select}
    <p class="text-sm">
      Selected: {vm.selected_utxo.length} outputs ·
      {display_sat(
        vm.selected_utxo_total_value,
      )}
    </p>
  {/if}
</div>

<style>
  .selection-column {
    width: 36px;
  }

  .selected-output {
    background: var(--app-primary-soft);
  }
</style>
