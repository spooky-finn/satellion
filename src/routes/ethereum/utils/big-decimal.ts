type BigDecimalValue = string | number | BigDecimal

export class BigDecimal {
	static #DECIMALS: number = 18 // Number of decimals on all instances
	static #ROUNDED: boolean = true // Numbers are truncated (false) or rounded (true)
	static #SHIFT: bigint = 10n ** BigInt(BigDecimal.#DECIMALS) // Derived constant
	static #fromBigInt: symbol = Symbol() // Secret to allow construction with given #n value
	#n!: bigint // the BigInt that will hold the BigDecimal's value multiplied by #SHIFT

	constructor(value: BigDecimalValue)
	constructor(value: bigint, convert: symbol)
	constructor(value: BigDecimalValue | bigint, convert?: symbol) {
		if (value instanceof BigDecimal) {
			this.#n = value.#n
			return
		}
		if (convert === BigDecimal.#fromBigInt) {
			// Can only be used within this class
			this.#n = value as bigint
			return
		}
		const [ints, decis] = String(value).split('.').concat('')
		this.#n =
			BigInt(
				ints +
					decis
						.padEnd(BigDecimal.#DECIMALS, '0')
						.slice(0, BigDecimal.#DECIMALS),
			) + BigInt(BigDecimal.#ROUNDED && decis[BigDecimal.#DECIMALS] >= '5')
	}

	add(num: BigDecimalValue): BigDecimal {
		return new BigDecimal(
			this.#n + new BigDecimal(num).#n,
			BigDecimal.#fromBigInt,
		)
	}

	subtract(num: BigDecimalValue): BigDecimal {
		return new BigDecimal(
			this.#n - new BigDecimal(num).#n,
			BigDecimal.#fromBigInt,
		)
	}

	static #divRound(dividend: bigint, divisor: bigint): BigDecimal {
		return new BigDecimal(
			dividend / divisor +
				(BigDecimal.#ROUNDED ? ((dividend * 2n) / divisor) % 2n : 0n),
			BigDecimal.#fromBigInt,
		)
	}

	multiply(num: BigDecimalValue): BigDecimal {
		return BigDecimal.#divRound(
			this.#n * new BigDecimal(num).#n,
			BigDecimal.#SHIFT,
		)
	}

	divide(num: BigDecimalValue): BigDecimal {
		return BigDecimal.#divRound(
			this.#n * BigDecimal.#SHIFT,
			new BigDecimal(num).#n,
		)
	}

	pow(exponent: number): BigDecimal {
		if (exponent === 0) {
			return new BigDecimal(1)
		}
		if (exponent < 0) {
			throw new Error('Negative exponent not supported')
		}
		let result = new BigDecimal(this)
		for (let i = 1; i < exponent; i++) {
			result = result.multiply(this)
		}
		return result
	}

	toFixed(precision: number): string {
		const str = this.toString()
		const [intPart, decPart = ''] = str.split('.')

		if (precision === 0) {
			return intPart
		}

		const paddedDec = decPart.padEnd(precision, '0').slice(0, precision)
		return `${intPart}.${paddedDec}`
	}

	toString(): string {
		let s = this.#n
			.toString()
			.replace('-', '')
			.padStart(BigDecimal.#DECIMALS + 1, '0')
		s = (
			s.slice(0, -BigDecimal.#DECIMALS) +
			'.' +
			s.slice(-BigDecimal.#DECIMALS)
		).replace(/(\.0*|0+)$/, '')
		return this.#n < 0 ? '-' + s : s
	}
}
