<script lang="ts">
  import { root_store } from '../../../view_model/root.svelte'
  import AccountSelector from '../../../components/AccountSelector.svelte'
  import Loading from '../../../components/Loading.svelte'
  import Modal from '../../../components/Modal.svelte'
  import NetworkStatus from '../../../components/NetworkStatus.svelte'
  import DisplaySat from '../../../components/DisplaySat.svelte'
  import AddressActions from './AddressActions.svelte'
  import PendingTransactions from './PendingTransactions.svelte'
  import ChildAddresses from './ChildAddresses.svelte'
  import FeeBump from './FeeBump.svelte'
  import UtxoList from './UtxoList.svelte'
  import Transfer from './Transfer.svelte'

  const btc = $derived(root_store.wallet.btc)
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
  <AddressActions {btc} />
  <DisplaySat value={btc.total_balance_sat} price={btc.usd_price} />
  {#if btc.utxo_list.sync.error}
    <p class="error" role="alert">{btc.utxo_list.sync.error}</p>
  {/if}
  <PendingTransactions {btc} />
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

<ChildAddresses {btc} />
<FeeBump {btc} />

<style>
  .account-status {
    gap: 8px 24px;
  }
</style>
