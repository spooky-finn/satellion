import { route } from './routes'

/** Reactive path state backed by the desktop webview's browser history. */
export class Router {
  path = $state(window.location.pathname)

  navigate(path: string, replace = false) {
    if (replace) window.history.replaceState(null, '', path)
    else if (path !== this.path) window.history.pushState(null, '', path)

    this.sync()
  }

  sync() {
    this.path = window.location.pathname
  }

  back() {
    if (window.history.length > 1) window.history.back()
    else this.navigate(route.unlock_wallet, true)
  }
}

export const router = new Router()
