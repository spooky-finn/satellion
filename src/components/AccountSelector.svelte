<script lang="ts">
  import { Popover, Portal } from '@skeletonlabs/skeleton-svelte'
  import type { AccountSelectorVM } from './account_selector'
  import { handle_err } from '../lib/handle_err'
  import AccountRename from './account_selector/AccountRename.svelte'
  import AccountOption from './account_selector/AccountOption.svelte'
  import AccountCreate from './account_selector/AccountCreate.svelte'
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
    <AccountRename {vm} />
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
                <AccountOption
                  {account}
                  active={account.index === vm.active_account}
                  onselect={select}
                  onrename={rename}
                />
              {/each}
              <AccountCreate {vm} oncreated={() => { open = false }} />
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
</style>
