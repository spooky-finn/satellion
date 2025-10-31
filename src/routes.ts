import {
  type RouteConfig,
  route as add_route,
  index
} from '@react-router/dev/routes'
import { useNavigate as useNavigateReactRouter } from 'react-router'

export const useNavigate = useNavigateReactRouter

const register = (url: string, path: string) => {
  return add_route(url.replace('/', ''), `routes/${path}`)
}

export const route = {
  unlock_wallet: '/',
  create_wallet: '/create_wallet',
  gen_mnemonic: '/gen_mnemonic',
  verify_mnemonic: '/verify_mnemonic',
  create_passphrase: '/create_passphrase',
  import_mnemonic: '/import_mnemonic',
  // chain specific routes
  ethereum: '/wallet/ethereum',
  bitcoin: '/wallet/bitcoin'
} as const

export default [
  index('routes/unlock_wallet.tsx'),
  register(route.create_wallet, 'create_wallet.tsx'),
  register(route.gen_mnemonic, 'mnemonic/gen.tsx'),
  register(route.verify_mnemonic, 'mnemonic/verify.tsx'),
  register(route.create_passphrase, 'mnemonic/create_passphrase.tsx'),
  register(route.import_mnemonic, 'mnemonic/import.tsx'),
  register(route.ethereum, 'ethereum/ethereum.tsx'),
  register(route.bitcoin, 'bitcoin.tsx')
] satisfies RouteConfig
