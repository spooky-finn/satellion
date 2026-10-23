<script lang="ts">
  import { commands, type BlockChain } from '../bindings'
  import { root_store } from '../view_model/root.svelte'
  import { route } from '../lib/routes'
  import { router } from '../lib/router.svelte'
  import { handle_err, unwrap_result } from '../lib/handle_err'
  import ThemeSwitcher from './ThemeSwitcher.svelte'
  import Icon from './Icon.svelte'

  let { hideLedgers = false }: { hideLedgers?: boolean } = $props()

  let switching = $state(false)

  async function switchChain(chain: BlockChain) {
    if (switching) return

    switching = true

    try {
      await commands.switchBlockchain(chain).then(unwrap_result)
      router.navigate(chain === 'Bitcoin' ? route.bitcoin : route.ethereum)
    } finally {
      switching = false
    }
  }

  function lock() {
    root_store.wallet.reset()
    root_store.unlock.reset()
    router.navigate(route.unlock_wallet, true)
  }
</script>

<nav class="wallet-navbar row between" aria-label="Main navigation">
  {#if !hideLedgers}
    <div class="chain-toggle" role="group" aria-label="Blockchain">
      {#each ['Bitcoin', 'Ethereum'] as chain (chain)}
        {@const active =
          router.path ===
          (chain === 'Bitcoin' ? route.bitcoin : route.ethereum)}
        <button
          class={['btn', active && 'active']}
          aria-pressed={active}
          disabled={switching}
          onclick={() => switchChain(chain as BlockChain).catch(handle_err)}
        >
          <img
            src={chain === 'Bitcoin' ? '/bitcoin.webp' : '/ethereum.webp'}
            width="22"
            height="22"
            alt=""
          >
          {chain}
        </button>
      {/each}
    </div>
  {/if}
  <div class="nav-actions row">
    <ThemeSwitcher />
    <button
      class={[
        'btn-icon',
        root_store.settings.config?.tor.enabled && 'tor-enabled',
      ]}
      aria-label="Settings"
      title={root_store.settings.config?.tor.enabled
        ? 'Settings · Tor enabled'
        : 'Settings'}
      onclick={() => router.navigate(route.settings)}
    >
      <Icon name="settings" />
    </button>
    <button
      class="btn-icon"
      aria-label="Lock wallet"
      title="Lock wallet"
      onclick={lock}
    >
      <Icon name="lock" />
    </button>
  </div>
</nav>

<style>
  .wallet-navbar {
    flex-wrap: nowrap;
    gap: 4px;
  }

  .chain-toggle {
    display: inline-flex;
    border-radius: 20px;
    overflow: hidden;
    flex-shrink: 0;
  }

  .chain-toggle .btn {
    border-radius: 0;
    font-weight: 600;
    color: var(--app-secondary);
    padding: 4px 12px;
    border: 0;
  }

  .chain-toggle img {
    height: 22px;
    width: auto;
  }

  .chain-toggle .btn.active {
    background: light-dark(#fbd98e, #704104);
    color: var(--app-primary-plain);
  }

  .chain-toggle .btn:hover {
    background: var(--app-primary-soft-hover);
  }

  .nav-actions {
    margin-left: auto;
    flex-wrap: nowrap;
    gap: 0;
  }

  .nav-actions :global(.btn-icon) {
    width: 42px;
  }

  .tor-enabled {
    color: light-dark(#1f7a40, #77c69a);
  }

  @media (max-width: 400px) {
    .chain-toggle .btn {
      padding-inline: 8px;
      gap: 4px;
    }

    .nav-actions :global(.btn-icon) {
      width: 32px;
    }
  }
</style>
