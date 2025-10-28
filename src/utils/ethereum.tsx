export function hexToDecimal(hex: string): string {
  if (!hex) return '0.0000'

  const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex
  const wei = BigInt('0x' + cleanHex)

  // 1 ETH = 10^18 wei
  const base = 10n ** 18n

  const intPart = wei / base
  const fracPart = wei % base

  if (wei === 0n) return '0'

  // Convert fractional part to fixed length (18 digits), then trim to 4
  let fracStr = fracPart.toString().padStart(18, '0').slice(0, 4)

  // Remove trailing zeros but leave at least one digit if intPart != 0
  fracStr = fracStr.replace(/0+$/, '')
  return fracStr.length > 0 ? `${intPart}.${fracStr}` : intPart.toString()
}
