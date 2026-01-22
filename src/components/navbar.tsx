import LockIcon from '@mui/icons-material/Lock'
import { Button, Tooltip } from '@mui/joy'
import { Link, useNavigate } from 'react-router'
import { type Chain, commands } from '../bindings'
import { route } from '../routes'
import { Row } from '../shortcuts'
import { AppMenu } from './menu'
import { notifier } from './notifier'
import { ThemeSwitcher } from './theme_switcher'

export const Navbar = ({ hideLedgers }: { hideLedgers?: boolean }) => {
	const navigate = useNavigate()
	return (
		<Row justifyContent={'space-between'} width={'100%'}>
			{hideLedgers !== true && (
				<Row gap={1}>
					<BlockchainLink
						to={route.bitcoin}
						src={new URL('/bitcoin.webp', import.meta.url).toString()}
						chain="Bitcoin"
					/>
					<BlockchainLink
						to={route.ethereum}
						src={new URL('/ethereum.webp', import.meta.url).toString()}
						chain="Ethereum"
					/>
				</Row>
			)}
			<Row ml="auto" gap={0}>
				<ThemeSwitcher />
				<AppMenu />
				<Tooltip title="Lock wallet" size="sm">
					<Button
						variant="plain"
						color="neutral"
						onClick={() => navigate(route.unlock_wallet)}
					>
						<LockIcon />
					</Button>
				</Tooltip>
			</Row>
		</Row>
	)
}

const BlockchainLink = (props: { to: string; src: string; chain: Chain }) => (
	<Link to={props.to}>
		<Button
			size="sm"
			color="neutral"
			variant="soft"
			startDecorator={
				<img src={props.src} alt={props.chain} width={'auto'} height={22} />
			}
			onClick={async () => {
				const res = await commands.chainSwitchEvent(props.chain)
				if (res.status === 'error') {
					notifier.err(res.error)
				}
			}}
		>
			{props.chain}
		</Button>
	</Link>
)
