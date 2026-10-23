<script lang="ts">
  import { onDestroy } from 'svelte'
  import { commands, MIN_PASSPHRASE_LEN } from '../bindings'
  import { root_store } from '../view_model/root.svelte'
  import { MnemonicStore } from './wallet/mnemonic_store.svelte'
  import { router } from '../lib/router.svelte'
  import { route } from '../lib/routes'
  import Loading from '../components/Loading.svelte'
  import Navbar from '../components/Navbar.svelte'
  import Icon from '../components/Icon.svelte'

  /** Local creation steps; recovery words and passphrases are cleared on unmount. */
  class CreateFlow {
    mode = $state<'choose' | 'generate' | 'verify' | 'passphrase' | 'import'>(
      'choose',
    )
    readonly mnemonic = new MnemonicStore()
    busy = $state(false)
    error = $state('')
    wordCount = $state(12)
    words = $state<string[]>(Array(12).fill(''))
    capsLock = $state(false)
    keyboardLang = $state('')
    selectedWord = $state(0)
    copied = $state(false)

    get importComplete() {
      return (
        this.words.length === this.wordCount &&
        this.words.every(
          word => !!word && root_store.mnemonic_wordlist.includes(word),
        )
      )
    }

    async generate() {
      this.busy = true
      this.error = ''
      this.copied = false

      try {
        await this.mnemonic.generate()

        if (this.mnemonic.mnemonic.length) this.mode = 'generate'
      } catch (error) {
        this.error = String(error)
      } finally {
        this.busy = false
      }
    }

    paste(event: ClipboardEvent) {
      const words =
        event.clipboardData
          ?.getData('text')
          .trim()
          .toLowerCase()
          .split(/\s+/) ?? []

      if ([12, 24].includes(words.length)) {
        event.preventDefault()
        this.wordCount = words.length
        this.words = words
      }
    }

    async copyMnemonic() {
      try {
        await navigator.clipboard.writeText(this.mnemonic.mnemonic.join(' '))
        this.copied = true
      } catch (error) {
        this.error = String(error)
      }
    }

    async save() {
      if (this.busy) return

      this.busy = true
      this.error = ''

      try {
        if (this.mode === 'import')
          this.mnemonic.set_mnemonic(
            this.words.map(word => word.trim().toLowerCase()),
          )

        await this.mnemonic.createWallet(
          this.mode === 'import' ? 'Import' : 'Generation',
        )
        router.navigate(route.unlock_wallet, true)
      } catch (error) {
        this.error = String(error)
      } finally {
        this.busy = false
      }
    }
  }

  const flow = new CreateFlow()
  const m = flow.mnemonic

  onDestroy(() => {
    m.mnemonic = []
    m.verification_words = {}
    m.passphrase_store.passphrase = ''
    m.passphrase_store.repeat_passphrase = ''
    flow.words = []
  })
</script>

<svelte:window
  onkeydown={(event) => {
    flow.capsLock = event.getModifierState('CapsLock')
    if (event.code.startsWith('Key'))
      flow.keyboardLang = navigator.language || 'en'
  }}
/>

{#snippet home()}
  <button
    class="btn preset-tonal-surface"
    type="button"
    aria-label="Back to wallets"
    title="Back to wallets"
    disabled={flow.busy}
    onclick={() => router.navigate(route.unlock_wallet)}
  >
    <Icon name="home" />
  </button>
{/snippet}
{#if flow.mode === 'verify'}
  <Navbar hideLedgers />
{/if}
<section class="create-page stack compact items-center">
  {#if flow.mode === 'choose'}
    <h1>Add wallet</h1>
    <div class="row flow-choice">
      <button
        class="btn preset-tonal-surface"
        disabled={flow.busy}
        onclick={() => {
          flow.mode = 'import'
        }}
      >
        Import
      </button>
      <button
        class="btn preset-tonal-surface"
        disabled={flow.busy}
        onclick={() => flow.generate()}
      >
        Generate
      </button>
    </div>
    {@render home()}
  {:else if flow.mode === 'generate'}
    <h1>Random secret private key</h1>
    <button
      class="soft-card mnemonic-phrase"
      title="Copy recovery phrase"
      aria-label="Copy recovery phrase"
      onclick={() => flow.copyMnemonic()}
    >
      {m.mnemonic.join(' ')}
    </button>
    {#if flow.copied}
      <p class="copied-text" role="status">Copied</p>
    {/if}
    <div class="row">
      {@render home()}
      <button
        class="btn btn-plain btn-neutral"
        disabled={flow.busy}
        onclick={() => flow.generate()}
      >
        Regenerate
      </button>
      <button
        class="btn preset-tonal-primary"
        disabled={flow.busy}
        onclick={() => {
          flow.mode = 'verify'
        }}
      >
        Continue
      </button>
    </div>
  {:else if flow.mode === 'verify'}
    <h1>Enter your mnemonic words</h1>
    <form
      class="stack compact items-center"
      onsubmit={(event) => {
        event.preventDefault()
        if (m.verify()) flow.mode = 'passphrase'
      }}
    >
      {#each m.verification_indices as index (index)}
        <label class="word-input row"
          ><span class="word-number" aria-hidden="true">{index + 1}</span
          ><span class="sr-only">Word {index + 1}</span>
          <input
            class="input input-sm"
            aria-label={`Word ${index + 1}`}
            autocomplete="off"
            spellcheck="false"
            list="mnemonic-wordlist"
            value={m.verification_words[index] ?? ''}
            oninput={(event) =>
              m.set_verification_words(
                index,
                event.currentTarget.value.toLowerCase(),
              )}
          >
        </label>
      {/each}
      <button class="btn preset-tonal-primary">Verify</button>
      {@render home()}
    </form>
    {#if m.verification_successfull === false}
      <p class="error" role="alert">
        Verification failed. Check your saved phrase.
      </p>
    {/if}
  {:else}
    <h1>{flow.mode === 'import' ? 'Import' : 'Imagine a passphrase'}</h1>
    <form
      class="stack compact items-center wallet-form"
      onsubmit={(event) => {
        event.preventDefault()
        flow.save()
      }}
    >
      <label class="field"
        ><span class="sr-only">Wallet name</span>
        <input
          class="input wallet-field"
          placeholder="Wallet name"
          autocomplete="off"
          bind:value={m.wallet_name}
          disabled={flow.busy}
          required
        ></label
      >
      {#if flow.mode === 'import'}
        <div class="soft-card stack compact mnemonic-card">
          <div class="row between">
            <p class="muted">Enter your mnemonic phrase</p>
            <label class="field"
              ><span class="sr-only">Recovery phrase length</span
              ><select
                class="select select-sm"
                value={flow.wordCount}
                disabled={flow.busy}
                onchange={(event) => {
                  flow.wordCount = Number(event.currentTarget.value)
                  flow.words = Array(flow.wordCount).fill('')
                }}
              >
                <option value={12}>12 words</option>
                <option value={24}>24 words</option>
              </select></label
            >
          </div>
          <div class="word-inputs">
            {#each flow.words as _, index (index)}
              <label class="word-input row"
                ><span class="word-number" aria-hidden="true">{index + 1}</span
                ><span class="sr-only">Word {index + 1}</span>
                <input
                  class="input input-sm"
                  aria-label={`Word ${index + 1}`}
                  type={flow.selectedWord === index ? 'text' : 'password'}
                  value={flow.words[index]}
                  autocomplete="off"
                  spellcheck="false"
                  disabled={flow.busy}
                  list={flow.selectedWord === index
                    ? 'mnemonic-wordlist'
                    : undefined}
                  onfocus={() => {
                    flow.selectedWord = index
                  }}
                  onpaste={(event) => flow.paste(event)}
                  oninput={(event) => {
                    flow.words[index] = event.currentTarget.value
                      .trim()
                      .toLowerCase()
                  }}
                >
              </label>
            {/each}
          </div>
        </div>
      {:else}
        <p class="passphrase-help">
          Use at least {MIN_PASSPHRASE_LEN} characters. Think of your wallet
          passphrase as an extra lock on your wallet. Your recovery words are
          the key, and the passphrase is the secret code that makes the key
          work. Keep the passphrase stored separately from your recovery words.
        </p>
      {/if}
      <label class="field"
        ><span class="sr-only">Passphrase</span>
        <input
          class="input wallet-field"
          type="password"
          placeholder="Passphrase"
          autocomplete="new-password"
          bind:value={m.passphrase_store.passphrase}
          minlength={MIN_PASSPHRASE_LEN}
          disabled={flow.busy}
          required
        ></label
      >
      <div class="row muted">
        <span class={flow.capsLock ? 'caps-warning' : ''}
          >Caps Lock: {flow.capsLock ? 'ON' : 'OFF'}</span
        ><span>Language: {flow.keyboardLang || 'unknown'}</span>
      </div>
      <label class="field"
        ><span class="sr-only">Repeat passphrase</span>
        <input
          class="input wallet-field"
          type="password"
          placeholder="Repeat passphrase"
          autocomplete="new-password"
          bind:value={m.passphrase_store.repeat_passphrase}
          disabled={flow.busy}
          required
        ></label
      >
      {#if m.passphrase_store.is_mismatch}
        <p class="error" role="alert">
          Passphrases must match and meet the minimum length.
        </p>
      {/if}
      <button
        class="btn preset-filled-primary-500"
        disabled={flow.busy ||
          !m.wallet_name.trim() ||
          !m.passphrase_store.is_passphrase_matched ||
          (flow.mode === 'import' && !flow.importComplete)}
      >
        {flow.busy
          ? 'Saving…'
          : flow.mode === 'import'
            ? 'Import'
            : 'Save'}
      </button>
      {@render home()}
    </form>
  {/if}
  {#if flow.busy}
    <Loading label="Preparing your wallet…" />
  {/if}
  {#if flow.error}
    <p role="alert" class="error">{flow.error}</p>
  {/if}
</section>
<datalist id="mnemonic-wordlist">
  {#each root_store.mnemonic_wordlist as word (word)}
    <option value={word}></option>
  {/each}
</datalist>

<style>
  .create-page {
    width: 100%;
    text-align: center;
  }

  .flow-choice {
    margin-block: 8px;
  }

  .wallet-form {
    width: 100%;
  }

  .wallet-field {
    width: 200px;
  }

  .mnemonic-phrase {
    max-width: min(100%, 900px);
    padding: 8px;
    font-size: 18px;
    line-height: 27px;
    text-align: left;
    cursor: copy;
  }

  .copied-text {
    color: light-dark(#1f7a40, #77c69a);
    font-size: 12px;
  }

  .word-inputs {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 4px;
    padding-block: 8px;
  }

  .mnemonic-card {
    max-width: min(100%, 900px);
  }

  .word-input {
    flex-wrap: nowrap;
    gap: 8px;
  }

  .word-number {
    width: 10px;
    text-align: right;
    font-size: 12px;
    color: var(--app-muted);
  }

  .word-input .input {
    width: 150px;
  }

  .passphrase-help {
    max-width: 550px;
    font-size: 14px;
    text-align: left;
  }

  .caps-warning {
    color: var(--app-warning-text);
  }

  @media (max-width: 380px) {
    .word-input .input {
      width: 125px;
    }
  }
</style>
