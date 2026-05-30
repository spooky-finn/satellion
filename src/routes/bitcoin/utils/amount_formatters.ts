/**
 * Convert satoshis (BigInt|string) to BTC string.
 * - Uses pure BigInt math
 * - Removes trailing zeros from fractional part
 * - Omits decimal point if fraction is zero
 */
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

/**
 * Convert satoshis to a USD string.
 *
 * Float math is fine here — for any displayable wallet amount (sats × price
 * stays well under Number.MAX_SAFE_INTEGER) and lets sub-dollar values like
 * network fees survive rounding instead of truncating to "$0".
 */
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

/**
 * Convert satoshis to a human-readable BTC string.
 * - < 100_000 satoshis: show as sats
 * - >= 100_000 satoshis: show in BTC with 4 decimal places
 *
 * @param satoshi - balance in satoshis as BigInt
 * @param decimals - satoshis per BTC (usually 100_000_000)
 */
export function display_sat(satoshi: bigint | string | number): string {
  const decimals: bigint = 100_000_000n
  const sats = BigInt(satoshi)
  if (sats < 100_000n) {
    return `${sats} sat`
  }
  return `₿${sat2btc(sats, decimals)}`
}
