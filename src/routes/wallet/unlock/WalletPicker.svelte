<script lang="ts">
  import ThemeSwitcher from '../../../components/ThemeSwitcher.svelte'
  import Icon from '../../../components/Icon.svelte'

  let {
    wallets,
    selected,
    disabled,
    onselect,
    onadd,
  }: {
    wallets: string[]
    selected: string | null
    disabled: boolean
    onselect: (name: string) => void
    onadd: () => void
  } = $props()
</script>

<nav class="login-wallets row" aria-label="Wallets">
  {#each wallets as name (name)}
    <button
      class={[
        'btn btn-sm',
        selected === name
          ? 'preset-filled-surface-500'
          : 'preset-outlined-surface-500',
      ]}
      title={name}
      aria-pressed={selected === name}
      {disabled}
      onclick={() => onselect(name)}
    >
      {name}
    </button>
  {/each}
  <button
    class="btn-icon"
    aria-label="Add a wallet"
    title="Add a wallet"
    {disabled}
    onclick={() => onadd()}
  >
    <Icon name="plus" />
  </button>
  <ThemeSwitcher iconOnly />
</nav>

<style>
  .login-wallets {
    position: fixed;
    inset-inline: 0;
    bottom: 16px;
    justify-content: center;
    padding-inline: 16px;
    max-height: 35dvh;
    overflow-y: auto;
  }

  .login-wallets .btn {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
  }
</style>
