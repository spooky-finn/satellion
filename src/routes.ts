import { useNavigate as useNavigateReactRouter } from 'react-router'

export const useNavigate = useNavigateReactRouter

export const route = {
  unlock_wallet: '/',
  create_wallet: '/create_wallet',
  gen_mnemonic: '/gen_mnemonic',
  verify_mnemonic: '/verify_mnemonic',
  create_passphrase: '/create_passphrase',
  import_mnemonic: '/import_mnemonic',
  // chain specific routes
  ethereum: '/wallet/ethereum',
  ethereum_send: '/wallet/ethereum/send',
  bitcoin: '/wallet/bitcoin'
} as const
