import { useNavigate as useNavigateReactRouter } from 'react-router'

export const useNavigate = useNavigateReactRouter

export const route = {
  unlock_wallet: '/',
  create_wallet: '/create_wallet',
  // chain specific routes
  ethereum: '/wallet/ethereum',
  ethereum_send: '/wallet/ethereum/send',
  bitcoin: '/wallet/bitcoin'
} as const
