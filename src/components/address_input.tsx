import { Input } from '@mui/joy'
import { makeAutoObservable, runInAction } from 'mobx'
import { observer } from 'mobx-react-lite'
import type { Result } from '../bindings'

export type AddressValidateRpc = (
  address: string,
) => Promise<Result<null, string>>

export class AddressInputVM {
  constructor(readonly validate_func: AddressValidateRpc) {
    makeAutoObservable(this)
  }

  val = ''
  set_val(address: string) {
    this.val = address
  }

  is_valid = false

  async validate() {
    const res = await this.validate_func(this.val)
    runInAction(() => {
      if (res.status === 'error') {
        this.is_valid = false
      } else {
        this.is_valid = true
      }
    })
  }

  reset() {
    this.val = ''
    this.is_valid = true
  }
}

export const AddressInput = observer(({ state }: { state: AddressInputVM }) => (
  <>
    <Input
      placeholder="Recipient address"
      sx={{ maxWidth: '500px' }}
      value={state.val}
      onChange={e => {
        state.set_val(e.target.value)
        state.validate()
      }}
      error={!!state.val && !state.is_valid}
    />
  </>
))
