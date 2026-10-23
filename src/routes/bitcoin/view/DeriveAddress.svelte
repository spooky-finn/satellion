<script lang="ts">
  import type { DeriveChildVM } from '../view_model/derive_child.vm.svelte'
  import { handle_err } from '../../../lib/handle_err'
  import Modal from '../../../components/Modal.svelte'
  import Copyable from '../../../components/Copyable.svelte'

  let {
    derive,
    onderive,
  }: { derive: DeriveChildVM; onderive: () => Promise<void> } = $props()
</script>

<Modal
  open={derive.is_open}
  title="Derive child address"
  nested
  busy={derive.loader.loading}
  onclose={() => derive.set_is_open(false)}
>
  <form
    class="stack compact"
    onsubmit={(event) => {
      event.preventDefault()
      onderive().catch(handle_err)
    }}
  >
    <label class="row"
      >Index<input
        class="input input-sm derive-index"
        type="number"
        min="0"
        max="2147483647"
        step="1"
        bind:value={derive.index}
        required
      ></label
    >
    <label class="field"
      ><span class="sr-only">Label</span>
      <input
        class="input input-sm derive-label"
        placeholder="Label"
        bind:value={derive.label}
        required
      ></label
    >
    <label class="field"
      ><span class="sr-only">Address type</span
      ><select class="select select-sm" bind:value={derive.proposal}>
        <option value="taproot">Taproot (BIP86)</option>
        <option value="segwit">SegWit (BIP84)</option>
      </select></label
    >
    <button
      class="btn preset-filled-primary-500"
      disabled={derive.loader.loading ||
        !derive.label?.trim() ||
        derive.index === null}
    >
      {derive.loader.loading ? 'Deriving…' : 'Derive'}
    </button>
  </form>
  {#if derive.address}
    <hr>
    <Copyable value={derive.address} label="Copy derived address" />
  {/if}
</Modal>

<style>
  .derive-index {
    width: 70px;
  }

  .derive-label {
    width: 200px;
  }
</style>
