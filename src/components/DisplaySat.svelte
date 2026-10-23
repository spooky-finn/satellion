<script lang="ts">
  import {
    display_sat,
    sat2usd,
  } from '../routes/bitcoin/utils/amount_formatters'

  let {
    value,
    price,
    label = 'Balance',
    digits,
  }: {
    value: string | number | bigint
    price: number
    label?: string
    digits?: number
  } = $props()
</script>

<p class="row balance-value">
  <span>{label ? `${label} ` : ''}{display_sat(value)} ~</span
  ><span>{sat2usd(value, price, digits)}</span>
</p>
