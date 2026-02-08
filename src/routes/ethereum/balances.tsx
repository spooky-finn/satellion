import { Remove } from '@mui/icons-material'
import AddIcon from '@mui/icons-material/Add'
import CachedIcon from '@mui/icons-material/Cached'
import {
	Button,
	Card,
	IconButton,
	Input,
	Modal,
	ModalClose,
	ModalDialog,
	Stack,
	Tooltip,
} from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { type ChangeEvent, useState } from 'react'
import { commands, type TokenBalance, type TokenType } from '../../bindings'
import { notifier } from '../../lib/notifier'
import { P, Progress, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'

export const BalanceCard = observer(() => (
	<Card variant="soft" size="sm">
		<Row alignItems="flex-start" justifyContent="space-between">
			<Balances />

			<Row alignItems="center" gap={1}>
				{root_store.ui_config?.eth_anvil && <AnvilSetBalanceButton />}
				<SpecifyTokenToTrack />
				<IconButton
					onClick={() => root_store.wallet.eth.getBalance()}
					variant="plain"
				>
					<CachedIcon />
				</IconButton>
			</Row>
		</Row>
	</Card>
))

const Balances = observer(() => {
	const { eth } = root_store.wallet
	const tokens = eth.balance.data?.tokens

	if (eth.balance.loading) return <Progress />
	if (!tokens?.length) return <P color="neutral">Tokens not found</P>

	const handleUntrack = async (address: string) => {
		await commands.ethUntrackToken(address)
		eth.removeTokenFromBalance(address)
	}

	return (
		<Stack spacing={0.5} sx={{ minWidth: 0 }}>
			{tokens.map(t => (
				<Token
					key={t.address}
					t={t}
					onUntrack={() => handleUntrack(t.address)}
				/>
			))}
		</Stack>
	)
})

const Token = ({
	t,
	onUntrack,
}: {
	t: TokenBalance
	onUntrack: () => void
}) => {
	const { symbol, balance } = t

	return (
		<Row
			alignItems="center"
			justifyContent="space-between"
			sx={{
				minWidth: 0,
				py: 0.25,
			}}
		>
			<P sx={{ fontWeight: 500, minWidth: 48 }}>{symbol}</P>

			<Tooltip title={balance} size="sm">
				<P
					sx={{
						flex: 1,
						textAlign: 'left',
						fontFamily: 'monospace',
						overflow: 'hidden',
						textOverflow: 'ellipsis',
						whiteSpace: 'nowrap',
						px: 1,
					}}
				>
					{formatBalance(balance)}
				</P>
			</Tooltip>

			{symbol !== 'ETH' && (
				<Tooltip title="Do not track" size="sm">
					<IconButton size="sm" onClick={onUntrack}>
						<Remove />
					</IconButton>
				</Tooltip>
			)}
		</Row>
	)
}

const AnvilSetBalanceButton = observer(() => {
	const handleSetAnvilBalance = async () => {
		const address = root_store.wallet.eth.address
		const res = await commands.ethAnvilSetInitialBalances(address)
		if (res.status === 'error') {
			notifier.err(res.error)
		} else {
			notifier.ok(res.data)
			root_store.wallet.eth.getBalance()
		}
	}

	return (
		<Tooltip title="Set initial Anvil balances (10 ETH + 9,999,999 USDT)">
			<IconButton
				onClick={handleSetAnvilBalance}
				variant="plain"
				color="primary"
			>
				<AddIcon />
			</IconButton>
		</Tooltip>
	)
})

const SpecifyTokenToTrack = observer(() => {
	const [open, setOpen] = useState(false)
	const [data, setData] = useState<TokenType | null>(null)

	const handleAddressInput = async (e: ChangeEvent<HTMLInputElement>) => {
		const address = e.target.value
		if (address.length < 40) {
			return
		}

		const res = await commands.ethTrackToken(address)
		if (res.status === 'error') {
			setOpen(false)
			notifier.err(res.error)
			return
		}

		setData(res.data)
	}

	return (
		<>
			<Button
				variant="plain"
				size="sm"
				onClick={() => {
					setOpen(true)
					setData(null)
				}}
				sx={{ fontWeight: 400 }}
				color="neutral"
			>
				Track token
			</Button>

			<Modal open={open} onClose={() => setOpen(false)}>
				<ModalDialog sx={{ pr: 6 }}>
					<ModalClose />
					{data ? (
						<P>{data.symbol} is now tracked</P>
					) : (
						<Input
							autoFocus
							placeholder="Token contract address"
							onChange={handleAddressInput}
						/>
					)}
				</ModalDialog>
			</Modal>
		</>
	)
})

function formatBalance(value: string, maxDecimals = 6) {
	if (!value.includes('.')) return value
	const [int, frac] = value.split('.')
	const trimmed = frac.slice(0, maxDecimals)
	return trimmed.length ? `${int}.${trimmed}` : int
}
