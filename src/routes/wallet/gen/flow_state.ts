import { makeAutoObservable } from 'mobx'

type Stage = 'select_mnemonic' | 'verify_mnemonic' | 'set_passphrase'

export class FlowState {
	constructor() {
		makeAutoObservable(this)
	}

	stage: Stage = 'select_mnemonic'
	set_stage(s: Stage) {
		this.stage = s
	}
}
