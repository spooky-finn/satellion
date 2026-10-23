<script lang="ts">
  import type { AccountSelectorVM } from './store.svelte'
  import { handle_err } from '../../lib/handle_err'

  let { vm }: { vm: AccountSelectorVM } = $props()
</script>

<form
  class="row"
  onsubmit={(event) => {
    event.preventDefault()
    vm.save_rename_account().catch(handle_err)
  }}
>
  <input
    class="input input-sm account-name"
    aria-label="Account name"
    bind:value={vm.rename.input}
    disabled={vm.rename.loader.loading}
    autocomplete="off"
    {@attach (element) => {
      element.focus()
      element.select()
    }}
    onblur={() => vm.save_rename_account().catch(handle_err)}
    onkeydown={(event) => {
      if (event.key === 'Escape') {
        event.preventDefault()
        event.stopPropagation()
        vm.cancel_rename_account()
      }
    }}
  >
</form>

<style>
  .account-name {
    width: 160px;
  }
</style>
