import { CssVarsProvider, Sheet } from '@mui/joy'
import { extendTheme } from '@mui/joy/styles'
import { Links, Meta, Outlet, Scripts, ScrollRestoration } from 'react-router'
import { NotifierOverlay } from './components/notifier/notification_overlay'

const theme = extendTheme({
  cssVarPrefix: 'mode-toggle',
  // @ts-ignore
  colorSchemeSelector: '.demo_mode-toggle-%s'
})

export default function Root() {
  return (
    <html>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
      </head>
      <CssVarsProvider theme={theme}>
        <body
          style={{
            backgroundColor: 'var(--mode-toggle-palette-background-surface)'
          }}
        >
          <Sheet>
            <Outlet />
            <NotifierOverlay />
            <ScrollRestoration />
            <Scripts />
          </Sheet>
        </body>
      </CssVarsProvider>
    </html>
  )
}
