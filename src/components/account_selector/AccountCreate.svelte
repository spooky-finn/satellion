<script lang="ts">
  import type { AccountSelectorVM } from './store.svelte'
  import { handle_err } from '../../lib/handle_err'
  import Icon from '../Icon.svelte'

  let { vm, oncreated }: { vm: AccountSelectorVM; oncreated: () => void } =
    $props()

  async function create(event: SubmitEvent) {
    event.preventDefault()
    await vm.create_account().catch(handle_err)

    if (!vm.create.name_input_visible) oncreated()
  }
</script>

<form class="row account-create" onsubmit={create}>
  {#if vm.create.name_input_visible}
    <input
      class="input input-sm"
      placeholder="Account name"
      aria-label="Account name"
      bind:value={vm.create.name_input}
      disabled={vm.create.loader.loading}
      autocomplete="off"
      {@attach (element) => {
        element.focus()
      }}
    >
  {/if}
  <button
    class="btn-icon"
    aria-label={vm.create.name_input_visible
      ? 'Create account'
      : 'Add account'}
    title="Add account"
    type={vm.create.name_input_visible ? 'submit' : 'button'}
    disabled={vm.create.loader.loading ||
      (vm.create.name_input_visible &&
        !vm.create.name_input.trim())}
    onclick={() => {
      if (!vm.create.name_input_visible)
        vm.create.set_name_input_visible(true)
    }}
  >
    <Icon name="plus" />
  </button>
</form>

<style>
  .account-create {
    padding: 4px 8px;
    flex-wrap: nowrap;
  }

  .account-create .btn-icon {
    margin-left: auto;
  }
</style>
