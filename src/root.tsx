import { Links, Meta, Outlet, Scripts, ScrollRestoration } from 'react-router'
import { NotifierOverlay } from './components/notifier/notification_overlay'

export default function Root() {
  return (
    <html>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
      </head>
      <body>
        <Outlet />
        <NotifierOverlay />
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  )
}
