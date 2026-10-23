/** Tracks an asynchronous request and ignores superseded completion state. */
export class Resource<T> {
  loading = $state(false)
  error = $state<string | undefined>(undefined)
  data = $state.raw<T | undefined>(undefined)
  private revision = 0

  constructor(private readonly fetcher: () => Promise<T>) {}

  async refresh(): Promise<T> {
    const revision = ++this.revision

    this.loading = true
    this.error = undefined

    try {
      const data = await this.fetcher()

      if (revision === this.revision) this.data = data

      return data
    } catch (error) {
      if (revision === this.revision) this.error = String(error)

      throw error
    } finally {
      if (revision === this.revision) this.loading = false
    }
  }
}
