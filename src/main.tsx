import React, { useLayoutEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter, Route, Routes } from 'react-router'
import { route } from './routes'
import Bitcoin from './routes/bitcoin/bitcoin'
import CreateWallet from './routes/create_wallet'
import Ethereum from './routes/ethereum/ethereum'
import CreatePassphrase from './routes/mnemonic/create_passphrase'
import GenMnemonic from './routes/mnemonic/gen'
import ImportMnemonic from './routes/mnemonic/import'
import VerifyMnemonic from './routes/mnemonic/verify'
import UnlockWallet from './routes/unlock_wallet'

import { CssVarsProvider } from '@mui/joy'
import { extendTheme } from '@mui/joy/styles'
import { NotifierOverlay } from './components/notifier/notification_overlay'

const theme = extendTheme({
  cssVarPrefix: 'mode-toggle',
  // @ts-ignore
  colorSchemeSelector: '.demo_mode-toggle-%s'
})

const App = () => {
  useLayoutEffect(() => {
    document.body.style.backgroundColor =
      'var(--mode-toggle-palette-background-surface)'
  })
  return (
    <Routes>
      <Route path={route.unlock_wallet} element={<UnlockWallet />} />
      <Route path={route.create_wallet} element={<CreateWallet />} />
      <Route path={route.gen_mnemonic} element={<GenMnemonic />} />
      <Route path={route.verify_mnemonic} element={<VerifyMnemonic />} />
      <Route path={route.create_passphrase} element={<CreatePassphrase />} />
      <Route path={route.import_mnemonic} element={<ImportMnemonic />} />
      <Route path={route.ethereum} element={<Ethereum />} />
      <Route path={route.bitcoin} element={<Bitcoin />} />
    </Routes>
  )
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <CssVarsProvider theme={theme}>
      <NotifierOverlay />
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </CssVarsProvider>
  </React.StrictMode>
)
