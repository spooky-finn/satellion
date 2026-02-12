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
 * Convert satoshis to USD string.
 * - Uses pure BigInt math (no float BTC conversion)
 * - Rounds down to whole dollars
 * - Formats using Intl.NumberFormat
 */
export function sat2usd(
  satoshi: bigint | string,
  usd_price: number | string,
): string {
  const sats = BigInt(satoshi)
  const decimals = 100_000_000n
  // Convert price to integer dollars (drop cents intentionally)
  const btc_usd_price_int = BigInt(Math.floor(Number(usd_price)))
  const usd_amount = (sats * btc_usd_price_int) / decimals
  return fmt_usd(usd_amount)
}

export const fmt_usd = (
  amount: number | bigint | string,
  locale: string = 'en-US',
): string =>
  new Intl.NumberFormat(locale, {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 0, // no cents
  }).format(Number(amount))

/**
 * Convert satoshis to a human-readable BTC string.
 * - < 100_000 satoshis: show as sats
 * - >= 100_000 satoshis: show in BTC with 4 decimal places
 *
 * @param satoshi - balance in satoshis as BigInt
 * @param decimals - satoshis per BTC (usually 100_000_000)
 */
export function display_sat(satoshi: bigint | string): string {
  const decimals: bigint = 100_000_000n
  const sats = BigInt(satoshi)
  if (sats < 100_000n) {
    return `${sats} sat`
  }
  return `₿${sat2btc(sats, decimals)}`
}
