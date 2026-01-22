import { makeAutoObservable } from 'mobx'

export class Loader<D> {
	loading = false
	start() {
		this.loading = true
	}
	stop() {
		this.loading = false
	}

	data: D | undefined
	set(data: D) {
		this.data = data
		this.loading = false
	}

	reset() {
		this.loading = false
		this.data = undefined
	}

	constructor() {
		makeAutoObservable(this)
	}
}
