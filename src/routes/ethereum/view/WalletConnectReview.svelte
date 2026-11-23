<script lang="ts">
  import type { WalletConnectReview } from '../../../bindings/eth'

  let {
    review,
    method,
    chain,
    chain_name,
  }: {
    review: WalletConnectReview
    method: string
    chain: number
    chain_name: (chain: number) => string
  } = $props()

  const transaction = $derived(method === 'eth_sendTransaction')
  const target_label = $derived(
    review.spender
      ? 'Token contract'
      : method === 'eth_signTypedData_v4'
        ? 'Verifying contract'
        : 'To',
  )

  const fields = $derived.by(() => {
    const rows: WalletConnectReview['fields'] = [['Network', chain_name(chain)]]
    if (review.switch_chain)
      rows.push(['Switch to', chain_name(review.switch_chain)])
    if (review.target) rows.push([target_label, review.target])
    if (review.value_eth !== null)
      rows.push(['Value', `${review.value_eth} ETH`])
    if (review.fee_eth !== null)
      rows.push(['Execution gas fee cap', `${review.fee_eth} ETH`])
    if (review.spender) rows.push(['Spender', review.spender])
    if (review.allowance) rows.push(['Allowance', review.allowance])
    return rows.concat(review.fields)
  })
</script>

<dl>
  {#each fields as [ label, value ] (label)}
    <div class="review-field">
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  {/each}
</dl>

{#if review.fee_eth !== null && (chain === 8453 || chain === 84532)}
  <p class="muted">Base L1 data fees may apply separately.</p>
{/if}

{#if review.details}
  <details open={!transaction}>
    <summary>{transaction ? 'Calldata' : 'Data to sign'}</summary>
    <pre>{review.details}</pre>
  </details>
{/if}

<style>
  .review-field {
    display: grid;
    grid-template-columns: minmax(8rem, 1fr) minmax(0, 3fr);
    gap: 4px 16px;
    padding-block: 8px;
    border-bottom: 1px solid var(--app-divider);
  }

  dt {
    color: var(--app-muted);
  }

  dd,
  pre {
    overflow-wrap: anywhere;
  }

  pre {
    max-height: 220px;
    margin-top: 8px;
    overflow: auto;
    white-space: pre-wrap;
  }

  @media (max-width: 480px) {
    .review-field {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
