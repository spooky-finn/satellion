import ArrowBackIcon from '@mui/icons-material/ArrowBack'
import { Alert, Divider, Input, Stack, Switch } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { type ReactNode, useState } from 'react'
import { useNavigate } from 'react-router'
import { commands } from '../bindings'
import { notifier } from '../lib/notifier'
import { route } from '../lib/routes'
import { B, P, Row } from '../shortcuts'
import { root_store } from '../view_model/root'

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

export const Settings = observer(() => {
  const navigate = useNavigate()
  const config = root_store.ui_config

  const [torEnabled, setTorEnabled] = useState(config?.tor_enabled ?? false)
  const [torProxy, setTorProxy] = useState(
    config?.tor_socks5_proxy ?? 'socks5://127.0.0.1:9050',
  )
  const [ethRpcUrl, setEthRpcUrl] = useState(
    config?.eth_rpc_url ?? 'https://ethereum-rpc.publicnode.com',
  )
  const [electrumServer, setElectrumServer] = useState(
    config?.btc_electrum_server ?? '',
  )
  const [omitPassphrase, setOmitPassphrase] = useState(
    config?.omit_passphrase_on_private_key ?? false,
  )
  const [saved, setSaved] = useState(false)

  async function save() {
    const res = await commands.setConfig({
      tor_enabled: torEnabled,
      tor_socks5_proxy: torProxy,
      eth_rpc_url: ethRpcUrl,
      btc_electrum_server: electrumServer.trim() || null,
      omit_passphrase_on_private_key: omitPassphrase,
    })
    if (res.status === 'error') {
      notifier.err(res.error)
      return
    }
    await root_store.request_config()
    setSaved(true)
  }

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

      <Section title="Network">
        <SettingRow
          label="Tor Network"
          description="Route connections through Tor for enhanced privacy"
        >
          <Switch
            checked={torEnabled}
            onChange={e => {
              setTorEnabled(e.target.checked)
              setSaved(false)
            }}
          />
        </SettingRow>

        <Stack gap={0.5}>
          <P level="body-sm" color="neutral">
            Tor SOCKS5 proxy
          </P>
          <Input
            value={torProxy}
            onChange={e => {
              setTorProxy(e.target.value)
              setSaved(false)
            }}
            disabled={!torEnabled}
            placeholder="socks5://127.0.0.1:9050"
            size="sm"
          />
          <P level="body-xs" color="neutral">
            Tor must be running locally. Bitcoin routes Electrum connections
            through the proxy; Ethereum routes the configured RPC URL through
            the proxy.
          </P>
        </Stack>
      </Section>

      <Section title="Ethereum">
        <Stack gap={0.5}>
          <P level="body-sm" color="neutral">
            RPC URL
          </P>
          <Input
            value={ethRpcUrl}
            onChange={e => {
              setEthRpcUrl(e.target.value)
              setSaved(false)
            }}
            placeholder="https://ethereum-rpc.publicnode.com"
            size="sm"
          />
        </Stack>

        {config?.eth_anvil && (
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

      <Section title="Bitcoin">
        <Stack gap={0.5}>
          <P level="body-sm" color="neutral">
            Electrum server
          </P>
          <Input
            value={electrumServer}
            onChange={e => {
              setElectrumServer(e.target.value)
              setSaved(false)
            }}
            placeholder="Leave blank to use default"
            size="sm"
          />
        </Stack>

        {config?.btc_regtest && (
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

      <Section title="Security">
        <SettingRow
          label="Omit passphrase from private key"
          description="Derive private keys without including the wallet passphrase"
        >
          <Switch
            checked={omitPassphrase}
            onChange={e => {
              setOmitPassphrase(e.target.checked)
              setSaved(false)
            }}
          />
        </SettingRow>
      </Section>

      {saved && (
        <Alert color="warning" size="sm">
          Restart the app to apply network changes.
        </Alert>
      )}

      <Row justifyContent="flex-end">
        <B size="sm" onClick={save}>
          Save
        </B>
      </Row>

      {root_store.wallet.name && (
        <Section title="Wallet">
          <SettingRow
            label="Forget wallet"
            description="Remove this wallet from the device. Your funds are safe as long as you have the seed phrase."
          >
            <B
              size="sm"
              color="danger"
              variant="soft"
              onClick={async () => {
                await root_store.wallet.forget(root_store.wallet.name!)
                navigate(route.unlock_wallet)
              }}
            >
              Forget
            </B>
          </SettingRow>
        </Section>
      )}
    </Stack>
  )
})
