import { makeAutoObservable } from 'mobx'

/**
 * Observable promise container for React 19 `use()` + Suspense.
 *
 * `promise` is observable, so when an `observer` reads it inside `use()`,
 * calling `refresh()` triggers a re-render and re-suspension on the new
 * promise. The fetcher is expected to throw on failure; the rejection
 * propagates through `use()` to the nearest ErrorBoundary.
 */
export class Resource<T> {
  private _promise: Promise<T> = new Promise<T>(() => {})

  constructor(private readonly _fetcher: () => Promise<T>) {
    makeAutoObservable(this)
  }

  get promise(): Promise<T> {
    return this._promise
  }

  refresh(): Promise<T> {
    this._promise = this._fetcher()
    return this._promise
  }
}
