import { CssVarsProvider } from '@mui/joy'
import { extendTheme } from '@mui/joy/styles'
import React, { useLayoutEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter, Route, Routes } from 'react-router'
import { NotifierOverlay } from './components/notifier/notification_overlay'
import { route } from './routes'
import Bitcoin from './routes/bitcoin/bitcoin'
import { CreateWallet } from './routes/create_wallet'
import { Ethereum } from './routes/ethereum/ethereum'
import { EthereumTransfer } from './routes/ethereum/transfer'
import UnlockWallet from './routes/unlock_wallet'

const theme = extendTheme({
	cssVarPrefix: 'mode-toggle',
	// @ts-expect-error
	colorSchemeSelector: '.demo_mode-toggle-%s',
	components: {
		JoyButton: {
			defaultProps: {
				size: 'sm',
			},
		},
	},
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
