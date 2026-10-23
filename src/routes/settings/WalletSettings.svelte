<script lang="ts">
  import type { SettingsVM } from '../../view_model/settings.svelte'
  import type { Wallet } from '../../view_model/wallet.svelte'
  import { notifier } from '../../lib/notifier'
  import { handle_err } from '../../lib/handle_err'

  let {
    s,
    wallet,
    onforget,
  }: { s: SettingsVM; wallet: Wallet; onforget: () => void } = $props()
</script>

<div class="stack">
  <p class="eyebrow">Wallet</p>
  <form
    class="row items-end"
    onsubmit={(event) => {
      event.preventDefault()
      s.rename_wallet()
        .then((ok) => {
          if (ok) notifier.ok('Wallet renamed')
        })
        .catch(handle_err)
    }}
  >
    <label class="field flex-1"
      >Wallet name<input
        class="input input-sm"
        bind:value={s.rename_draft}
        required
      ></label
    ><button
      class="btn preset-filled-primary-500"
      disabled={!s.rename_draft.trim() ||
        s.rename_draft.trim() === wallet.name}
    >
      Rename
    </button>
  </form>
  <div class="row between forget-row">
    <div class="forget-copy">
      <p>Forget wallet</p>
      <p class="muted">
        Remove this wallet from the device. Your funds are safe as long as you
        have the seed phrase.
      </p>
    </div>
    <button
      class="btn preset-tonal-error"
      onclick={(event) => {
        event.currentTarget.focus()
        onforget()
      }}
    >
      Forget
    </button>
  </div>
</div>

<style>
  .forget-row {
    flex-wrap: nowrap;
  }

  .forget-copy {
    flex: 1;
  }
</style>
