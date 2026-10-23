<script lang="ts">
  import { Popover, Portal } from '@skeletonlabs/skeleton-svelte'
  import type { AccountSelectorVM } from './account_selector'
  import { handle_err } from '../lib/handle_err'
  import Copyable from './Copyable.svelte'
  import Icon from './Icon.svelte'

  let { vm }: { vm: AccountSelectorVM } = $props()

  let open = $state(false)

  async function select(index: number) {
    open = false
    await vm.handle_account_switch(index).catch(handle_err)
  }

  function rename(account: AccountSelectorVM['accounts'][number]) {
    open = false
    vm.start_rename_account(account)
  }

  async function create(event: SubmitEvent) {
    event.preventDefault()
    await vm.create_account().catch(handle_err)

    if (!vm.create.name_input_visible) open = false
  }

  function move(event: KeyboardEvent) {
    if (
      event.target instanceof HTMLInputElement ||
      !['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)
    )
      return

    const buttons = Array.from(
      event.currentTarget instanceof HTMLElement
        ? event.currentTarget.querySelectorAll<HTMLButtonElement>(
            '[data-account]',
          )
        : [],
    )

    if (!buttons.length) return

    event.preventDefault()

    const current = buttons.indexOf(document.activeElement as HTMLButtonElement)
    const next =
      event.key === 'Home'
        ? 0
        : event.key === 'End'
          ? buttons.length - 1
          : (current + (event.key === 'ArrowDown' ? 1 : -1) + buttons.length) %
            buttons.length

    buttons[next].focus()
  }
</script>

<div class="account-selector row">
  <span>Account</span>
  {#if vm.rename.account !== null}
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
  {:else}
    <Popover
      {open}
      autoFocus={false}
      onOpenChange={(event) => {
        open = event.open
        if (!open) vm.create.reset()
      }}
      positioning={{ placement: 'bottom-start', gutter: 4 }}
    >
      <Popover.Trigger
        class="btn btn-plain account-trigger"
        aria-label="Account"
        disabled={vm.account_loader.loading}
      >
        <span class="account-current"
          >{vm.account_loader.loading
            ? 'Loading…'
            : (vm.active_account_item?.name ?? 'Select account')}</span
        >
        <Icon name="chevron" size={16} />
      </Popover.Trigger>
      {#if open}
        <Portal>
          <Popover.Positioner class="z-30">
            <Popover.Content
              class="account-menu stack compact"
              aria-label="Accounts"
              onkeydown={move}
              {@attach (element) => {
                element
                  .querySelector<HTMLElement>('[aria-pressed="true"]')
                  ?.focus()
              }}
            >
              {#each vm.accounts as account (account.index)}
                <div
                  class={[
                    'account-option row',
                    account.index === vm.active_account && 'selected',
                  ]}
                >
                  <button
                    class="account-pick stack compact"
                    data-account
                    type="button"
                    aria-label={`Select account ${account.name}`}
                    aria-pressed={account.index === vm.active_account}
                    onclick={() => select(account.index)}
                  >
                    <span class="row"
                      ><small>[{account.index}]</small>{account.name}</span
                    >
                  </button>
                  <button
                    class="btn-icon account-edit"
                    aria-label={`Rename account ${account.name}`}
                    title="Rename account"
                    onclick={() => rename(account)}
                  >
                    <Icon name="edit" />
                  </button>
                  {#if account.address}
                    <div class="account-address">
                      <Copyable
                        value={account.address}
                        label={`Copy ${account.name} address`}
                        small
                      />
                    </div>
                  {/if}
                </div>
              {/each}
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
            </Popover.Content>
          </Popover.Positioner>
        </Portal>
      {/if}
    </Popover>
  {/if}
</div>

<style>
  .account-selector {
    gap: 4px;
    flex-wrap: nowrap;
    flex-shrink: 1;
    max-width: 100%;
  }

  :global(.account-trigger) {
    font-weight: 400;
    padding-inline: 8px;
    gap: 12px;
    max-width: 200px;
  }

  .account-current {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .account-name {
    width: 160px;
  }

  :global(.account-menu) {
    min-width: 240px;
    max-width: min(360px, calc(100vw - 24px));
    padding: 4px;
    border: 1px solid var(--app-border);
    background: var(--app-popup);
    border-radius: 6px;
    box-shadow: 0 4px 16px rgb(0 0 0 / 0.12);
    gap: 0;
  }

  .account-option {
    position: relative;
    padding: 4px 8px;
    gap: 0;
    border-radius: 4px;
  }

  .account-option:hover,
  .account-option:focus-within {
    background: var(--app-neutral-soft);
  }

  .account-option.selected {
    background: var(--app-primary-soft);
  }

  .account-pick {
    flex: 1;
    text-align: left;
    gap: 0;
    padding-right: 16px;
  }

  .account-pick small {
    font-size: 12px;
    color: var(--app-muted);
  }

  .account-address {
    flex-basis: 100%;
    font-weight: 600;
  }

  .account-edit {
    opacity: 0;
  }

  .account-option:hover .account-edit,
  .account-option:focus-within .account-edit {
    opacity: 1;
  }

  .account-create {
    padding: 4px 8px;
    flex-wrap: nowrap;
  }

  .account-create .btn-icon {
    margin-left: auto;
  }

  @media (hover: none) {
    .account-edit {
      opacity: 1;
    }
  }
</style>
