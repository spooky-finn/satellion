<script lang="ts">
  import type { Account } from './store.svelte'
  import Copyable from '../Copyable.svelte'
  import Icon from '../Icon.svelte'

  let {
    account,
    active,
    onselect,
    onrename,
  }: {
    account: Account
    active: boolean
    onselect: (index: number) => void
    onrename: (account: Account) => void
  } = $props()
</script>

<div class={['account-option row', active && 'selected']}>
  <button
    class="account-pick stack compact"
    data-account
    type="button"
    aria-label={`Select account ${account.name}`}
    aria-pressed={active}
    onclick={() => onselect(account.index)}
  >
    <span class="row"><small>[{account.index}]</small>{account.name}</span>
  </button>
  <button
    class="btn-icon account-edit"
    aria-label={`Rename account ${account.name}`}
    title="Rename account"
    onclick={() => onrename(account)}
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

<style>
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

  @media (hover: none) {
    .account-edit {
      opacity: 1;
    }
  }
</style>
