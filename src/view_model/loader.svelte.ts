/** Reactive loading state for manually managed requests and their last result. */
export class Loader<D> {
  loading = $state(false)

  start() {
    this.loading = true
  }

  stop() {
    this.loading = false
  }

  data = $state.raw<D | undefined>(undefined)

  set(data: D) {
    this.data = data
    this.loading = false
  }

  reset() {
    this.loading = false
    this.data = undefined
  }
}
