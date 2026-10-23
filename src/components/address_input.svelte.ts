import type { Result } from '../bindings'

export type AddressValidateRpc = (
  address: string,
) => Promise<Result<null, string>>

/** Validates the current address while ignoring replies for earlier input. */
export class AddressInputVM {
  val = $state('')
  is_valid = $state(false)
  validating = $state(false)
  private revision = 0

  constructor(readonly validate_func: AddressValidateRpc) {}

  set_val(address: string) {
    this.val = address.trim()
    this.is_valid = false
    this.validating = false
    this.revision++
  }

  async validate() {
    const revision = ++this.revision
    const address = this.val

    if (!address) return

    this.validating = true

    try {
      const result = await this.validate_func(address)

      if (revision === this.revision) this.is_valid = result.status === 'ok'
    } finally {
      if (revision === this.revision) this.validating = false
    }
  }

  reset() {
    this.revision++
    this.val = ''
    this.is_valid = false
    this.validating = false
  }
}
