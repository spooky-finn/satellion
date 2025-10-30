const PRECISION = 4

export function weiToEth(wei: bigint): string {
  const base = 10n ** 18n
  const intPart = wei / base
  const fracPart = wei % base
  if (wei === 0n) return '0'
  // Convert fractional part to fixed length (18 digits), then trim to 4
  let fracStr = fracPart.toString().padStart(18, '0').slice(0, PRECISION)
  // Remove trailing zeros but leave at least one digit if intPart != 0
  fracStr = fracStr.replace(/0+$/, '')
  return fracStr.length > 0 ? `${intPart}.${fracStr}` : intPart.toString()
}
