<script lang="ts">
  import type { AddressInputVM } from './address_input.svelte'
  import { handle_err } from '../lib/handle_err'

  let {
    state,
    disabled = false,
    onchange = () => {},
  }: {
    state: AddressInputVM
    disabled?: boolean
    onchange?: () => void
  } = $props()
</script>

<label class="field recipient-field"
  ><span class="sr-only">Recipient address</span>
  <input
    class="input"
    placeholder="Recipient address"
    style:width={`${Math.max(20, state.val.length + 2)}ch`}
    style:max-width="100%"
    autocomplete="off"
    spellcheck="false"
    value={state.val}
    {disabled}
    oninput={(event) => {
      state.set_val(event.currentTarget.value)
      onchange()
      state.validate().catch(handle_err)
    }}
    aria-invalid={!!state.val && !state.is_valid && !state.validating}
  >
</label>
{#if state.val && !state.is_valid && !state.validating}
  <p class="error">Enter a valid recipient address.</p>
{/if}

<style>
  .recipient-field {
    width: fit-content;
    max-width: 100%;
  }
</style>
