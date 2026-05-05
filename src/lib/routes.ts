import { useNavigate as useNavigateReactRouter } from 'react-router'

export const useNavigate = useNavigateReactRouter

export const route = {
  unlock_wallet: '/',
  create_wallet: '/create_wallet',
  ethereum: '/wallet/ethereum',
  bitcoin: '/wallet/bitcoin',
} as const
