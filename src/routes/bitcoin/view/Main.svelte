<script lang="ts">
  import { root_store } from '../../../view_model/root.svelte'
  import { DeriveChildVM } from '../view_model/derive_child.vm.svelte'
  import { FeeBumpState } from '../view_model/fee_bump.vm.svelte'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { handle_err } from '../../../lib/handle_err'
  import AccountSelector from '../../../components/AccountSelector.svelte'
  import Copyable from '../../../components/Copyable.svelte'
  import Loading from '../../../components/Loading.svelte'
  import Modal from '../../../components/Modal.svelte'
  import UtxoList from './UtxoList.svelte'
  import Transfer from './Transfer.svelte'
  import NetworkStatus from '../../../components/NetworkStatus.svelte'
  import DisplaySat from '../../../components/DisplaySat.svelte'

  const btc = $derived(root_store.wallet.btc)
  const derive = new DeriveChildVM()

  async function openDerive() {
    derive.set_is_open(true)
    await derive.next_unused_key_index()
  }

  async function deriveAddress() {
    await derive.derive()
    await btc.child_list.fetch()
  }
</script>

<section class="stack compact" aria-label="Bitcoin wallet">
  <h1 class="sr-only">Bitcoin wallet</h1>
  <div class="row between account-status">
    <AccountSelector vm={btc.account_selector} />
    <NetworkStatus
      height={btc.chain_tip?.number}
      hash={btc.chain_tip?.hash}
      price={btc.usd_price}
    />
  </div>
  {#if btc.account_info.loading}
    <Loading label="Refreshing account…" />
  {/if}
  {#if btc.account_info.error}
    <p class="error" role="alert">{btc.account_info.error}</p>
  {/if}
  {#if btc.address}
    <div class="soft-card stack compact">
      <div class="font-semibold">
        <Copyable value={btc.address} label="Copy address" />
      </div>
      <div class="row">
        <button
          class="btn preset-tonal-primary"
          onclick={(event) => {
            event.currentTarget.focus()
            btc.child_list.set_open(true)
          }}
        >
          Child addresses
        </button>
        <button
          class="btn preset-tonal-primary"
          onclick={(event) => {
            event.currentTarget.focus()
            btc.utxo_list.open()
          }}
        >
          Utxo
        </button>
        <button
          class="btn preset-filled-primary-500"
          disabled={btc.account_selector.account_loader.loading}
          onclick={(event) => {
            event.currentTarget.focus()
            btc.transfer.reset()
            btc.utxo_list.reset()
            btc.transfer.set_open(true)
          }}
        >
          Send
        </button>
        <button
          class="btn btn-plain ml-auto"
          disabled={btc.discovery_loader.loading}
          onclick={() => btc.discover_wallet().catch(handle_err)}
        >
          {btc.discovery_loader.loading ? 'Discovering…' : 'Discover'}
        </button>
      </div>
      {#if btc.discovery_loader.data}
        <p class="muted">
          Discovery found {btc.discovery_loader.data.accounts.length} accounts,
          added
          {btc.discovery_loader.data.paths_added}
          paths and
          {btc
            .discovery_loader.data.utxos_added}
          UTXOs
        </p>
      {/if}
    </div>
  {/if}
  <DisplaySat value={btc.total_balance_sat} price={btc.usd_price} />
  {#if btc.utxo_list.sync.error}
    <p class="error" role="alert">{btc.utxo_list.sync.error}</p>
  {/if}
  {#if btc.utxo_list.pending_parent_txs.length}
    <div class="soft-card warning-card stack compact">
      <h2 class="pending-title">Pending transactions</h2>
      <p class="muted">
        Outputs from these txs are still in the mempool. Bump the fee with a
        CPFP child to help them confirm.
      </p>
      {#each btc.utxo_list.pending_parent_txs as tx (tx.parent_tx_id)}
        <div class="row between">
          <div class="stack gap-0">
            <Copyable
              value={tx.parent_tx_id}
              label="Copy pending transaction ID"
              small
            />
            <DisplaySat
              value={tx.value_sat}
              price={btc.usd_price}
              label="Pending"
              digits={2}
            />
          </div>
          <button
            class="btn preset-filled-primary-500"
            onclick={(event) => {
              event.currentTarget.focus()
              btc.fee_bump.open(tx.parent_tx_id)
            }}
          >
            Bump fee
          </button>
        </div>
      {/each}
    </div>
  {/if}
</section>
<Transfer {btc} />
<Modal
  open={btc.utxo_list.is_open}
  title="Unspent transaction outputs"
  fullscreen
  onclose={() => btc.utxo_list.close()}
>
  <UtxoList vm={btc.utxo_list} price={btc.usd_price} />
</Modal>
<Modal
  open={btc.child_list.is_open}
  title="Child addresses"
  fullscreen
  onclose={() => btc.child_list.set_open(false)}
>
  <div class="row">
    <button
      class="btn preset-filled-primary-500"
      onclick={(event) => {
        event.currentTarget.focus()
        openDerive().catch(handle_err)
      }}
    >
      Derive
    </button>
    <button
      class="btn btn-plain"
      onclick={() => btc.child_list.fetch().catch(handle_err)}
    >
      Refresh
    </button>
  </div>
  {#if btc.child_list.loader.loading}
    <Loading label="Loading addresses…" />
  {/if}
  {#if btc.child_list.addresses.length}
    <div class="table-wrap">
      <table class="table">
        <thead>
          <tr>
            <th>Label</th>
            <th>Derivation path</th>
            <th>Address</th>
          </tr>
        </thead>
        <tbody>
          {#each btc.child_list.addresses as address (address.path)}
            <tr>
              <td>{address.label}</td>
              <td class="mono text-xs">{address.path}</td>
              <td>
                <Copyable value={address.address} label="Copy child address" />
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <p>No addresses derived yet.</p>
  {/if}
  <Modal
    open={derive.is_open}
    title="Derive child address"
    nested
    busy={derive.loader.loading}
    onclose={() => derive.set_is_open(false)}
  >
    <form
      class="stack compact"
      onsubmit={(event) => {
        event.preventDefault()
        deriveAddress().catch(handle_err)
      }}
    >
      <label class="row"
        >Index<input
          class="input input-sm derive-index"
          type="number"
          min="0"
          max="2147483647"
          step="1"
          bind:value={derive.index}
          required
        ></label
      >
      <label class="field"
        ><span class="sr-only">Label</span>
        <input
          class="input input-sm derive-label"
          placeholder="Label"
          bind:value={derive.label}
          required
        ></label
      >
      <label class="field"
        ><span class="sr-only">Address type</span
        ><select class="select select-sm" bind:value={derive.proposal}>
          <option value="taproot">Taproot (BIP86)</option>
          <option value="segwit">SegWit (BIP84)</option>
        </select></label
      >
      <button
        class="btn preset-filled-primary-500"
        disabled={derive.loader.loading ||
          !derive.label?.trim() ||
          derive.index === null}
      >
        {derive.loader.loading ? 'Deriving…' : 'Derive'}
      </button>
    </form>
    {#if derive.address}
      <hr>
      <Copyable value={derive.address} label="Copy derived address" />
    {/if}
  </Modal>
</Modal>
<Modal
  open={!!btc.fee_bump.parent_tx_id}
  title="Bump fee (CPFP)"
  fullscreen
  busy={btc.fee_bump.state === FeeBumpState.Sending}
  onclose={() => btc.fee_bump.close()}
>
  {#if btc.fee_bump.result}
    <h2 class="result-title">Child transaction broadcast</h2>
    <div class="stack gap-1">
      <p class="text-sm">Child txid</p>
      <Copyable
        value={btc.fee_bump.result.child_tx_id}
        label="Copy child transaction ID"
        small
      />
    </div>
    <div class="stack gap-1">
      <p class="text-sm">Child fee</p>
      <DisplaySat
        value={btc.fee_bump.result.child_fee}
        price={btc.usd_price}
        label=""
        digits={2}
      />
    </div>
    <hr>
    <div class="row">
      <button
        class="btn preset-tonal-primary"
        onclick={() =>
          openUrl(
            `https://mempool.space/tx/${btc.fee_bump.result!.child_tx_id}`,
          ).catch(handle_err)}
      >
        Open explorer
      </button>
      <button class="btn btn-plain" onclick={() => btc.fee_bump.close()}>
        Close
      </button>
    </div>
  {:else}
    <div class="stack gap-1">
      <p class="text-sm">Parent transaction</p>
      <Copyable
        value={btc.fee_bump.parent_tx_id ?? ''}
        label="Copy parent transaction ID"
        small
      />
    </div>
    <p class="muted">
      Spends every output of the parent that belongs to this wallet into a new
      self-send. Miners with package-aware mempool policies will pull the parent
      in once the child pays enough.
    </p>
    <form
      class="stack"
      onsubmit={(event) => {
        event.preventDefault()
        btc.fee_bump.submit().catch(handle_err)
      }}
    >
      <label class="input-unit"
        ><span class="sr-only">Target fee rate (sat/vB)</span>
        <input
          class="input"
          type="number"
          placeholder="Fee rate"
          min="0.1"
          step="0.1"
          bind:value={btc.fee_bump.fee_rate_sat_vb}
          required
        ><span>sat/vB</span></label
      >
      <div class="row">
        <button
          type="button"
          class="btn btn-plain"
          disabled={btc.fee_bump.state === FeeBumpState.Sending}
          onclick={() => btc.fee_bump.close()}
        >
          Cancel
        </button>
        <button
          class="btn preset-filled-primary-500"
          disabled={btc.fee_bump.state === FeeBumpState.Sending}
        >
          {btc.fee_bump.state === FeeBumpState.Sending
            ? 'Broadcasting…'
            : 'Broadcast CPFP'}
        </button>
      </div>
    </form>
  {/if}
  {#if btc.fee_bump.error}
    <p class="error" role="alert">{btc.fee_bump.error}</p>
  {/if}
</Modal>

<style>
  .account-status {
    gap: 8px 24px;
  }

  .pending-title,
  .result-title {
    font-size: 14px;
    line-height: 20px;
    letter-spacing: 0;
    font-weight: 500;
  }

  .result-title {
    font-size: 16px;
    font-weight: 400;
  }

  .derive-index {
    width: 70px;
  }

  .derive-label {
    width: 200px;
  }
</style>
