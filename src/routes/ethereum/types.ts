export interface Chain {
  block_number: number
  block_hash: string
  base_fee_per_gas: number | null
}

export interface TokenBalance {
  token_symbol: string
  balance: string
  decimals: number
  ui_precision: number
}

export interface Balance {
  wei: string
  tokens: TokenBalance[]
}
