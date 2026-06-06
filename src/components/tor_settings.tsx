import { useState } from 'react'
import { Alert, Input, Stack, Switch, Typography } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { commands } from '../bindings'
import { notifier } from '../lib/notifier'
import { root_store } from '../view_model/root'
import { B, FullScreenModal, P, Row } from '../shortcuts'

export const TorSettings = observer(
  ({ open, onClose }: { open: boolean; onClose: () => void }) => {
    const config = root_store.ui_config
    const [enabled, setEnabled] = useState(config?.tor_enabled ?? false)
    const [proxy, setProxy] = useState(
      config?.tor_socks5_proxy ?? 'socks5://127.0.0.1:9050',
    )
    const [saved, setSaved] = useState(false)

    async function save() {
      const res = await commands.setTorConfig(enabled, proxy)
      if (res.status === 'error') {
        notifier.err(res.error)
        return
      }
      setSaved(true)
    }

    return (
      <FullScreenModal open={open} onClose={onClose}>
        <Stack gap={2}>
          <P level="title-md">Tor Network Settings</P>

          <Row alignItems="center" justifyContent="space-between">
            <P>Route connections through Tor</P>
            <Switch
              checked={enabled}
              onChange={e => {
                setEnabled(e.target.checked)
                setSaved(false)
              }}
            />
          </Row>

          <Stack gap={0.5}>
            <Typography level="body-sm" color="neutral">
              SOCKS5 proxy address
            </Typography>
            <Input
              value={proxy}
              onChange={e => {
                setProxy(e.target.value)
                setSaved(false)
              }}
              disabled={!enabled}
              placeholder="socks5://127.0.0.1:9050"
              size="sm"
            />
            <Typography level="body-xs" color="neutral">
              Tor must be running locally. Bitcoin routes Electrum connections
              through the proxy; Ethereum routes the configured RPC URL through
              the proxy.
            </Typography>
          </Stack>

          {saved && (
            <Alert color="warning" size="sm">
              Restart the app to apply Tor settings.
            </Alert>
          )}

          <Row justifyContent="flex-end" gap={1}>
            <B variant="plain" color="neutral" size="sm" onClick={onClose}>
              Cancel
            </B>
            <B size="sm" onClick={save}>
              Save
            </B>
          </Row>
        </Stack>
      </FullScreenModal>
    )
  },
)
