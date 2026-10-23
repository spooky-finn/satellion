<script lang="ts">
  import Modal from '../../../components/Modal.svelte'
  import type { TokenTracker } from '../view_model/token_tracker.svelte'

  let { vm }: { vm: TokenTracker } = $props()
</script>

<Modal
  open={vm.is_open}
  title="Track a token"
  busy={vm.busy}
  onclose={() => {
    vm.close()
  }}
>
  {#if vm.tracked}
    <p role="status">{vm.tracked.symbol} is now tracked.</p>
    <button
      class="btn preset-filled-primary-500"
      onclick={() => {
        vm.close()
      }}
    >
      Done
    </button>
  {:else}
    <form
      class="stack"
      onsubmit={(event) => {
        event.preventDefault()
        vm.track()
      }}
    >
      <label class="field"
        ><span class="sr-only">Token contract address</span>
        <input
          class="input"
          placeholder="Token contract address"
          autocomplete="off"
          spellcheck="false"
          bind:value={vm.tokenAddress}
          disabled={vm.busy}
          required
        ></label
      >
      <button
        class="btn preset-filled-primary-500"
        disabled={vm.busy || !vm.tokenAddress.trim()}
      >
        {vm.busy ? 'Tracking…' : 'Track token'}
      </button>
    </form>
  {/if}
  {#if vm.error}
    <p class="error" role="alert">{vm.error}</p>
  {/if}
</Modal>
