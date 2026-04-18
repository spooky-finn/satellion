import { CssVarsProvider } from '@mui/joy'
import { listen } from '@tauri-apps/api/event'
import React, { useLayoutEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter, Route, Routes, useNavigate } from 'react-router'
import { NotifierOverlay } from './lib/notifier/notification_overlay'
import { theme } from './lib/notifier/theme'
import { route } from './routes'
import BitcoinWallet from './routes/bitcoin/interface/main'
import { CreateWallet } from './routes/create_wallet'
import { EthereumWallet } from './routes/ethereum/interface/main'
import { EthereumTransfer } from './routes/ethereum/interface/transfer'
import UnlockWallet from './routes/unlock_wallet'

const App = () => {
  const navigate = useNavigate()

  useLayoutEffect(() => {
    listen('session_expired', () => {
      navigate(route.unlock_wallet)
    })

    document.body.style.backgroundColor =
      'var(--mode-toggle-palette-background-surface)'
  })

  return (
    <Routes>
      <Route path={route.unlock_wallet} element={<UnlockWallet />} />
      <Route path={route.create_wallet} element={<CreateWallet />} />
      <Route path={route.ethereum} element={<EthereumWallet />} />
      <Route path={route.ethereum_send} element={<EthereumTransfer />} />
      <Route path={route.bitcoin} element={<BitcoinWallet />} />
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
  </React.StrictMode>,
)
