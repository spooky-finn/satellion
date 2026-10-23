<script lang="ts">
  import type { EthereumWalletVM } from '../view_model/wallet.vm.svelte'
  import type { FeeMode } from '../../../bindings/eth'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { handle_err } from '../../../lib/handle_err'
  import Modal from '../../../components/Modal.svelte'
  import AddressInput from '../../../components/AddressInput.svelte'
  import HoldToSend from '../../../components/HoldToSend.svelte'
  import Copyable from '../../../components/Copyable.svelte'

  let { eth }: { eth: EthereumWalletVM } = $props()

  const t = $derived(eth.transfer)
  const selected = $derived(
    eth.balance.data?.tokens.find(token => token.address === t.token),
  )
  const locked = $derived(t.sending || t.is_estimating || !!t.estimation)

  async function send() {
    await t.execute()

    if (t.tx_hash) await eth.getBalance()
  }
</script>

<Modal
  open={t.is_open}
  title="Transact on Ethereum"
  fullscreen
  busy={t.sending || t.is_estimating}
  onclose={() => t.set_open(false)}
>
  {#if t.tx_hash}
    <h2>Transaction sent</h2>
    <Copyable value={t.tx_hash} />
    <button
      class="btn preset-tonal-primary"
      onclick={() =>
        openUrl(`https://etherscan.io/tx/${t.tx_hash}`).catch(handle_err)}
    >
      Open explorer
    </button><button class="btn btn-plain" onclick={() => t.reset()}>
      Send another
    </button>
  {:else}
    <form
      class="stack compact"
      onsubmit={(event) => {
        event.preventDefault()
        t.estimate()
      }}
    >
      <AddressInput
        state={t.address}
        disabled={locked}
        onchange={() => t.invalidate()}
      />
      <label class="field"
        ><span class="sr-only">Token</span
        ><select
          class="select"
          aria-label="Token"
          value={t.token ?? ''}
          disabled={locked}
          onchange={(event) =>
            t.set_token(event.currentTarget.value || undefined)}
        >
          <option value="" disabled>Select a token</option>
          {#each eth.tokens_with_balance as token (token.address)}
            <option value={token.address}>{token.symbol}</option>
          {/each}
        </select></label
      >
      {#if selected}
        <p class="muted">
          Balance: {selected.balance}
          {selected.symbol}
        </p>
      {/if}
      <label class="field"
        ><span class="sr-only">Amount</span>
        <input
          class="input amount-input"
          placeholder="Amount"
          type="text"
          inputmode="decimal"
          value={t.amount ?? ''}
          disabled={locked}
          oninput={(event) =>
            t.set_amount(event.currentTarget.value || undefined)}
          required
        ></label
      >
      {#if t.amount_error}
        <p class="error">{t.amount_error}</p>
      {/if}
      <div class="segmented" role="group" aria-label="Fee speed">
        {#each [{ value: 'Minimal', label: 'Slow' }, { value: 'Standard', label: 'Standard' }, { value: 'Increased', label: 'Fast' }] as mode (mode.value)}
          <button
            type="button"
            class="btn"
            aria-pressed={t.fee_mode === mode.value}
            disabled={locked}
            onclick={() => t.set_fee_mode(mode.value as FeeMode)}
          >
            {mode.label}
          </button>
        {/each}
      </div>
      {#if !t.estimation}
        <button
          class="btn preset-filled-primary-500"
          disabled={t.disabled || t.is_estimating}
        >
          {t.is_estimating ? 'Estimating…' : 'Estimate'}
        </button>
      {/if}
    </form>
    {#if t.estimation}
      <div class="stack compact">
        <h2>Review your transfer</h2>
        <p>{t.amount} {selected?.symbol}</p>
        <p class="muted">Estimated gas: {t.estimation.estimated_gas}</p>
        <p class="muted">
          Network fee ceiling: {t.estimation.fee_ceiling} gwei · ~${t.estimation.fee_in_usd.toFixed(
            2,
          )}
        </p>
        <div class="row">
          <button
            class="btn btn-plain"
            disabled={t.sending}
            onclick={() => t.invalidate()}
          >
            Edit transfer
          </button><HoldToSend busy={t.sending} onsend={send} />
        </div>
      </div>
    {/if}
  {/if}
  {#if t.error}
    <p role="alert" class="error">{t.error}</p>
  {/if}
</Modal>

<style>
  .amount-input {
    width: 202px;
    max-width: 100%;
  }

  h2 {
    font-size: 16px;
    letter-spacing: 0;
  }
</style>
