import ArrowBackIcon from '@mui/icons-material/ArrowBack'
import SettingsIcon from '@mui/icons-material/Settings'
import { Alert, Divider, Input, Stack, Switch } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { type ReactNode, useEffect } from 'react'
import { useNavigate } from 'react-router'
import { notifier } from '../lib/notifier'
import { route } from '../lib/routes'
import { B, LinkButton, P, Row } from '../shortcuts'
import { root_store } from '../view_model/root'

export const SettingsLink = observer(() => {
  const torEnabled = root_store.settings.tor_enabled
  return (
    <LinkButton
      to={route.settings}
      variant="plain"
      color={torEnabled ? 'success' : 'neutral'}
    >
      <SettingsIcon />
    </LinkButton>
  )
})

const Section = ({
  title,
  children,
}: {
  title: string
  children: ReactNode
}) => (
  <Stack gap={2}>
    <P
      level="body-xs"
      color="neutral"
      sx={{ textTransform: 'uppercase', letterSpacing: '0.08em' }}
    >
      {title}
    </P>
    {children}
    <Divider />
  </Stack>
)

const SettingRow = ({
  label,
  description,
  children,
}: {
  label: string
  description?: string
  children: ReactNode
}) => (
  <Row alignItems="center" justifyContent="space-between">
    <Stack gap={0.25}>
      <P level="body-md">{label}</P>
      {description && (
        <P level="body-xs" color="neutral">
          {description}
        </P>
      )}
    </Stack>
    {children}
  </Row>
)

const TorSection = observer(() => {
  const s = root_store.settings
  return (
    <Section title="Network">
      <SettingRow
        label="Tor Network"
        description="Route connections through Tor for enhanced privacy"
      >
        <Switch
          checked={s.tor_enabled}
          onChange={e => s.set_tor_enabled(e.target.checked)}
        />
      </SettingRow>

      <Stack gap={0.5}>
        <P level="body-sm" color="neutral">
          Tor SOCKS5 proxy
        </P>
        <Input
          value={s.tor_proxy}
          onChange={e => s.set_tor_proxy(e.target.value)}
          disabled={!s.tor_enabled}
          placeholder="socks5://127.0.0.1:9050"
          size="sm"
        />
        <P level="body-xs" color="neutral">
          Tor must be running locally. Bitcoin routes Electrum connections
          through the proxy; Ethereum routes the configured RPC URL through the
          proxy.
        </P>
      </Stack>
    </Section>
  )
})

const EthereumSection = observer(() => {
  const s = root_store.settings
  return (
    <Section title="Ethereum">
      <Stack gap={0.5}>
        <P level="body-sm" color="neutral">
          RPC URL
        </P>
        <Input
          value={s.eth_rpc_url}
          onChange={e => s.set_eth_rpc_url(e.target.value)}
          placeholder="https://ethereum-rpc.publicnode.com"
          size="sm"
        />
      </Stack>

      {s.eth_anvil && (
        <SettingRow
          label="Anvil"
          description="Local testnet mode is active for this session"
        >
          <P level="body-sm" color="success">
            Active
          </P>
        </SettingRow>
      )}
    </Section>
  )
})

const BitcoinSection = observer(() => {
  const s = root_store.settings
  return (
    <Section title="Bitcoin">
      <Stack gap={0.5}>
        <P level="body-sm" color="neutral">
          Electrum server
        </P>
        <Input
          value={s.electrum_server}
          onChange={e => s.set_electrum_server(e.target.value)}
          placeholder="Leave blank to use default"
          size="sm"
        />
      </Stack>

      {s.btc_regtest && (
        <SettingRow
          label="Regtest"
          description="Local regtest mode is active for this session"
        >
          <P level="body-sm" color="warning">
            Active
          </P>
        </SettingRow>
      )}
    </Section>
  )
})

const SecuritySection = observer(() => {
  const s = root_store.settings
  return (
    <Section title="Security">
      <SettingRow
        label="Omit passphrase from private key"
        description="Derive private keys without including the wallet passphrase"
      >
        <Switch
          checked={s.omit_passphrase}
          onChange={e => s.set_omit_passphrase(e.target.checked)}
        />
      </SettingRow>

      <SettingRow
        label="Session timeout"
        description="Lock the wallet after this many minutes of inactivity"
      >
        <Input
          type="number"
          value={s.session_timeout_mins}
          onChange={e => {
            const v = parseInt(e.target.value, 10)
            if (!isNaN(v) && v > 0) s.set_session_timeout_mins(v)
          }}
          slotProps={{ input: { min: 1, max: 1440 } }}
          size="sm"
          sx={{ width: 80 }}
          endDecorator={
            <P level="body-xs" color="neutral">
              min
            </P>
          }
        />
      </SettingRow>
    </Section>
  )
})

const WalletSection = observer(() => {
  const navigate = useNavigate()
  const { wallet, settings } = root_store

  if (!wallet.name) return null

  return (
    <Section title="Wallet">
      <Stack gap={0.5}>
        <P level="body-sm" color="neutral">
          Wallet name
        </P>
        <Row gap={1}>
          <Input
            value={settings.rename_draft}
            onChange={e => settings.set_rename_draft(e.target.value)}
            size="sm"
            sx={{ flex: 1 }}
          />
          <B
            size="sm"
            disabled={
              !settings.rename_draft.trim() ||
              settings.rename_draft.trim() === wallet.name
            }
            onClick={async () => {
              const ok = await root_store.settings.rename_wallet()
              if (ok) notifier.ok('Wallet renamed')
            }}
          >
            Rename
          </B>
        </Row>
      </Stack>

      <SettingRow
        label="Forget wallet"
        description="Remove this wallet from the device. Your funds are safe as long as you have the seed phrase."
      >
        <B
          size="sm"
          color="danger"
          variant="soft"
          onClick={async () => {
            await wallet.forget(wallet.name!)
            navigate(route.unlock_wallet)
          }}
        >
          Forget
        </B>
      </SettingRow>
    </Section>
  )
})

export const Settings = observer(() => {
  const navigate = useNavigate()
  const s = root_store.settings

  useEffect(() => {
    root_store.settings.load_settings()
  }, [])

  return (
    <Stack gap={3} maxWidth={480} p={1}>
      <Row alignItems="center" gap={1}>
        <B
          variant="plain"
          color="neutral"
          size="sm"
          onClick={() => navigate(-1)}
        >
          <ArrowBackIcon />
        </B>
        <P level="title-lg">Settings</P>
      </Row>

      <TorSection />
      <EthereumSection />
      <BitcoinSection />
      <SecuritySection />

      {s.saved && (
        <Alert color="warning" size="sm">
          Restart the app to apply network changes.
        </Alert>
      )}

      <Row justifyContent="flex-end">
        <B
          size="sm"
          onClick={async () => {
            await s.save()
            await root_store.settings.load_settings()
          }}
        >
          Save
        </B>
      </Row>

      <WalletSection />
    </Stack>
  )
})
