/** Format satoshis as BTC using integer math, without trailing decimal zeros. */
export function sat2btc(
  satoshi: bigint | string,
  decimals: bigint = 100_000_000n,
): string {
  const sats = BigInt(satoshi)
  const whole = sats / decimals
  const fraction = sats % decimals

  if (fraction === 0n) {
    return whole.toString()
  }

  let fraction_str = fraction.toString().padStart(8, '0')

  fraction_str = fraction_str.replace(/0+$/, '')

  return `${whole}.${fraction_str}`
}

/** Format a USD display value; transaction amounts remain in satoshis. */
export function sat2usd(
  satoshi: bigint | string | number,
  usd_price: number | string,
  fraction_digits?: number,
): string {
  const usd_amount = (Number(satoshi) * Number(usd_price)) / 1e8

  return fmt_usd(usd_amount, fraction_digits)
}

export const fmt_usd = (
  amount: number | bigint | string,
  fraction_digits: number = 0,
): string =>
  new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: fraction_digits,
  }).format(Number(amount))

/** Show small balances in sats and balances from 100,000 sats in BTC. */
export function display_sat(satoshi: bigint | string | number): string {
  const decimals: bigint = 100_000_000n
  const sats = BigInt(satoshi)

  if (sats < 100_000n) {
    return `${sats} sat`
  }

  return `₿${sat2btc(sats, decimals)}`
}
