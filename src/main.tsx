import { CssVarsProvider } from '@mui/joy'
import { listen } from '@tauri-apps/api/event'
import React, { useLayoutEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter, Route, Routes, useNavigate } from 'react-router'
import { NotifierOverlay } from './lib/notifier/notification_overlay'
import { theme } from './lib/notifier/theme'
import { route } from './routes'
import Bitcoin from './routes/bitcoin/bitcoin'
import { CreateWallet } from './routes/create_wallet'
import { Ethereum } from './routes/ethereum/ethereum'
import { EthereumTransfer } from './routes/ethereum/transfer'
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
      <Route path={route.ethereum} element={<Ethereum />} />
      <Route path={route.ethereum_send} element={<EthereumTransfer />} />
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
  </React.StrictMode>,
)
