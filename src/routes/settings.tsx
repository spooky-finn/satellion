import ArrowBackIcon from '@mui/icons-material/ArrowBack'
import FingerprintIcon from '@mui/icons-material/Fingerprint'
import SettingsIcon from '@mui/icons-material/Settings'
import { Alert, Input, Stack, Switch } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { DynamicConfigForm } from '../components/config_form'
import { useKeyDown } from '../components/use_key_down'
import { notifier } from '../lib/notifier'
import { route } from '../lib/routes'
import { B, LinkButton, P, Row } from '../shortcuts'
import { root_store } from '../view_model/root'

export const SettingsLink = observer(() => {
  const torEnabled = root_store.settings.config?.tor.enabled ?? false
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

const WalletSection = observer(() => {
  const navigate = useNavigate()
  const { wallet, settings } = root_store

  if (!wallet.name) return null

  return (
    <Stack gap={2}>
      <P
        level="body-xs"
        color="neutral"
        sx={{ textTransform: 'uppercase', letterSpacing: '0.08em' }}
      >
        Wallet
      </P>

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

      <Row alignItems="center" justifyContent="space-between">
        <Stack gap={0.25}>
          <P level="body-md">Forget wallet</P>
          <P level="body-xs" color="neutral">
            Remove this wallet from the device. Your funds are safe as long as
            you have the seed phrase.
          </P>
        </Stack>
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
      </Row>
    </Stack>
  )
})

export const Settings = observer(() => {
  const navigate = useNavigate()
  const s = root_store.settings

  useEffect(() => {
    s.load_settings()
  }, [])

  useKeyDown('Escape', () => navigate(-1))

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

      {s.biometric_supported && (
        <Row alignItems="center" justifyContent="space-between">
          <Stack gap={0.25}>
            <Row alignItems="center" gap={0.5}>
              <FingerprintIcon fontSize="sm" />
              <P level="body-md">Unlock with Touch ID</P>
            </Row>
            <P level="body-xs" color="neutral">
              Stores the passphrase in the macOS Keychain.
            </P>
          </Stack>
          <Switch
            checked={s.biometric_enabled}
            disabled={s.biometric_busy}
            onChange={e => s.toggle_biometric(e.target.checked)}
          />
        </Row>
      )}

      {s.config && s.schema && (
        <DynamicConfigForm
          root={s.schema}
          schema={s.schema}
          values={s.config as Record<string, unknown>}
          onChangePath={(path, value) => s.set_value(path, value)}
        />
      )}

      {s.restart_hint && (
        <Alert color="warning" size="sm">
          Restart the app to apply network changes.
        </Alert>
      )}

      <Row justifyContent="flex-end">
        <B
          size="sm"
          fullWidth
          onClick={async () => {
            await s.save()
            await s.load_settings()
          }}
        >
          Save
        </B>
      </Row>

      <WalletSection />
    </Stack>
  )
})
