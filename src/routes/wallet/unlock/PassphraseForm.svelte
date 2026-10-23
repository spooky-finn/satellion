<script lang="ts">
  import type { Unlock } from '../../../view_model/unlock.svelte'

  let { unlock, onsubmit }: { unlock: Unlock; onsubmit: () => Promise<void> } =
    $props()
</script>

<form
  class="stack items-center"
  onsubmit={(event) => {
    event.preventDefault()
    onsubmit()
  }}
>
  <label class="field">
    <span class="sr-only">Passphrase</span>
    <input
      class="input input-soft login-passphrase"
      type="password"
      placeholder="Passphrase"
      autocomplete="off"
      bind:value={unlock.passphrase}
      {@attach (element) => {
        element.focus()
      }}
    >
  </label>
  <button
    type="submit"
    class="sr-only"
    tabindex="-1"
    disabled={!unlock.passphrase}
  >
    Unlock wallet
  </button>
</form>

<style>
  .login-passphrase {
    width: 200px;
  }
</style>
