use alloy::{
    primitives::{
        Address, B256, TxKind, U256, address,
        utils::{format_ether, format_units},
    },
    providers::Provider,
    rpc::types::{TransactionInput, TransactionRequest},
    sol,
    sol_types::SolCall,
};
use alloy_provider::DynProvider;
use serde_json::Value;

use super::{WalletConnectReview, settings::network_name};

sol! {
    function deposit(
        bytes32 depositor, bytes32 recipient, bytes32 inputToken, bytes32 outputToken,
        uint256 inputAmount, uint256 outputAmount, uint256 destinationChainId,
        bytes32 exclusiveRelayer, uint32 quoteTimestamp, uint32 fillDeadline,
        uint32 exclusivityParameter, bytes message
    );
    function depositV3(
        address depositor, address recipient, address inputToken, address outputToken,
        uint256 inputAmount, uint256 outputAmount, uint256 destinationChainId,
        address exclusiveRelayer, uint32 quoteTimestamp, uint32 fillDeadline,
        uint32 exclusivityParameter, bytes message
    );
    function depositNative(
        address spokePool, address depositor, bytes32 recipient, address inputToken,
        uint256 inputAmount, bytes32 outputToken, uint256 outputAmount, uint256 destinationChainId,
        bytes32 exclusiveRelayer, uint32 quoteTimestamp, uint32 fillDeadline,
        uint32 exclusivityParameter, bytes message
    );
    function approve(address spender, uint256 value);
    function supportsInterface(bytes4 interfaceId) returns (bool);
}

// Across deployment registry: across-protocol/contracts/broadcast/deployed-addresses.json.
fn across_contracts(chain: u32) -> (Address, Address) {
    let mainnet_periphery = address!("4e9ef0a5fe749079dd11e68aad5d75fccfaaf9af");
    let testnet_periphery = address!("10d8b8daa26d307489803e10477de69c0492b610");
    match chain {
        1 => (
            address!("5c7bcd6e7de5423a257d81b442095a1a6ced35c5"),
            mainnet_periphery,
        ),
        8453 => (
            address!("09aea4b2242abc8bb4bb78d537a67a245a7bec64"),
            mainnet_periphery,
        ),
        11155111 => (
            address!("5ef6c01e11889d86803e0b23e3cb3f9e9d97b662"),
            testnet_periphery,
        ),
        84532 => (
            address!("82b564983ae7274c86695917bbf8c99ecb6f0f8f"),
            testnet_periphery,
        ),
        _ => unreachable!("validated WalletConnect chain"),
    }
}

fn wrapped_eth(chain: u32) -> Address {
    match chain {
        1 => address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
        11155111 => address!("fff9976782d46cc05630d1f6ebab18b2324d6b14"),
        _ => address!("4200000000000000000000000000000000000006"),
    }
}

fn token_info(chain: u32, token: Address) -> Option<(&'static str, u8)> {
    if token == wrapped_eth(chain) {
        return Some(("WETH", 18));
    }
    let usdc = match chain {
        1 => address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
        8453 => address!("833589fcd6edb6e08f4c7c32d4f71b54bda02913"),
        11155111 => address!("1c7d4b196cb0c7b01d743fbc6116a902379c7238"),
        84532 => address!("036cbd53842c5426634e7929541ec2318f3dcf7e"),
        _ => return None,
    };
    (token == usdc).then_some(("USDC", 6))
}

fn token_amount(chain: u32, token: Address, amount: U256) -> String {
    match token_info(chain, token) {
        Some((symbol, decimals)) => {
            let amount = format_units(amount, decimals).expect("valid token decimals");
            format!("{amount} {symbol} ({token})")
        }
        None => format!("{amount} base units ({token})"),
    }
}

struct Deposit {
    depositor: Address,
    recipient: Address,
    input_token: Address,
    output_token: Address,
    input_amount: U256,
    output_amount: U256,
    destination: U256,
}

fn evm_address(word: B256) -> Result<Address, String> {
    if word[..12] != [0; 12] {
        return Err("Across address is outside the supported EVM address format".into());
    }
    Ok(Address::from_word(word))
}

fn decode_deposit<C: SolCall>(input: &[u8]) -> Result<C, String> {
    // Across app and integrator attribution suffixes are outside the ABI payload.
    let end = input.len() - (input.len() - 4) % 32;
    if !matches!(
        &input[end..],
        [] | [0x73, 0xc0, 0xde] | [0x1d, 0xc0, 0xde, _, _]
    ) {
        return Err("Unknown Across calldata suffix".into());
    }
    let call = C::abi_decode_validate(&input[..end]).map_err(|_| "Invalid Across calldata")?;
    if call.abi_encode() != input[..end] {
        return Err("Noncanonical Across calldata".into());
    }
    Ok(call)
}

fn plain_deposit(message: &[u8]) -> Result<(), String> {
    if message.is_empty() {
        Ok(())
    } else {
        Err("Across deposits that execute destination calldata are unsupported".into())
    }
}

fn across_deposit(chain: u32, target: Address, input: &[u8]) -> Result<Option<Deposit>, String> {
    let (pool, periphery) = across_contracts(chain);
    if target == pool && input.starts_with(&depositCall::SELECTOR) {
        let call = decode_deposit::<depositCall>(input)?;
        plain_deposit(&call.message)?;
        return Ok(Some(Deposit {
            depositor: evm_address(call.depositor)?,
            recipient: evm_address(call.recipient)?,
            input_token: evm_address(call.inputToken)?,
            output_token: evm_address(call.outputToken)?,
            input_amount: call.inputAmount,
            output_amount: call.outputAmount,
            destination: call.destinationChainId,
        }));
    }
    if target == pool && input.starts_with(&depositV3Call::SELECTOR) {
        let call = decode_deposit::<depositV3Call>(input)?;
        plain_deposit(&call.message)?;
        return Ok(Some(Deposit {
            depositor: call.depositor,
            recipient: call.recipient,
            input_token: call.inputToken,
            output_token: call.outputToken,
            input_amount: call.inputAmount,
            output_amount: call.outputAmount,
            destination: call.destinationChainId,
        }));
    }
    if target == periphery && input.starts_with(&depositNativeCall::SELECTOR) {
        let call = decode_deposit::<depositNativeCall>(input)?;
        plain_deposit(&call.message)?;
        if call.spokePool != pool {
            return Err("Across deposit does not target the official SpokePool".into());
        }
        return Ok(Some(Deposit {
            depositor: call.depositor,
            recipient: evm_address(call.recipient)?,
            input_token: call.inputToken,
            output_token: evm_address(call.outputToken)?,
            input_amount: call.inputAmount,
            output_amount: call.outputAmount,
            destination: call.destinationChainId,
        }));
    }
    Ok(None)
}

async fn supports_erc721(provider: &DynProvider, token: Address) -> Result<bool, String> {
    let call = supportsInterfaceCall {
        interfaceId: [0x80, 0xac, 0x58, 0xcd].into(),
    };
    let request = TransactionRequest::default()
        .to(token)
        .input(call.abi_encode().into());
    let result = provider
        .call(request)
        .await
        .map_err(|_| "Could not verify the token standard")?;
    supportsInterfaceCall::abi_decode_returns_validate(&result)
        .map_err(|_| "Could not verify the token standard".into())
}

pub(super) async fn review_transaction(
    provider: &DynProvider,
    tx: &TransactionRequest,
    review: &mut WalletConnectReview,
    chain: u32,
    address: Address,
) -> Result<(), String> {
    let target = *tx
        .to
        .and_then(|kind| match kind {
            alloy::primitives::TxKind::Call(address) => Some(address),
            _ => None,
        })
        .as_ref()
        .expect("normalized transaction recipient");
    let input = tx.input.input().expect("normalized calldata");
    let value = tx.value.unwrap_or_default();
    review.target = Some(target.to_checksum(None));
    if input.is_empty() {
        let code = provider
            .get_code_at(target)
            .await
            .map_err(|_| "Could not check the ETH recipient")?;
        if !code.is_empty() {
            return Err("WalletConnect ETH transfers to contracts require a supported call".into());
        }
        review.title = "Send ETH".into();
        return Ok(());
    }
    if let Some(deposit) = across_deposit(chain, target, input)? {
        let destination: u32 = deposit
            .destination
            .try_into()
            .map_err(|_| "Unsupported bridge destination")?;
        if !matches!(
            (chain, destination),
            (1, 8453) | (8453, 1) | (11155111, 84532) | (84532, 11155111)
        ) {
            return Err("Bridge destination is outside the enabled network pair".into());
        }
        if deposit.depositor != address
            || deposit.recipient.is_zero()
            || deposit.input_token.is_zero()
            || deposit.output_token.is_zero()
            || deposit.input_amount.is_zero()
            || deposit.output_amount.is_zero()
            || (!value.is_zero()
                && (value != deposit.input_amount || deposit.input_token != wrapped_eth(chain)))
        {
            return Err(
                "Across deposit has inconsistent assets, amount, recipient or refund address"
                    .into(),
            );
        }
        review.title = "Bridge via Across".into();
        review.fields.extend([
            (
                "Final recipient".into(),
                deposit.recipient.to_checksum(None),
            ),
            (
                "Destination".into(),
                format!("{} ({destination})", network_name(destination)),
            ),
            (
                "Send".into(),
                if value.is_zero() {
                    token_amount(chain, deposit.input_token, deposit.input_amount)
                } else {
                    format!("{} ETH", format_ether(value))
                },
            ),
            (
                "Quoted output".into(),
                if deposit.output_token == wrapped_eth(destination) {
                    format!(
                        "{} ETH (WETH for contract recipients)",
                        format_ether(deposit.output_amount)
                    )
                } else {
                    token_amount(destination, deposit.output_token, deposit.output_amount)
                },
            ),
        ]);
        return Ok(());
    }
    if value.is_zero() && input.starts_with(&approveCall::SELECTOR) {
        let call =
            approveCall::abi_decode_validate(input).map_err(|_| "Invalid approval calldata")?;
        if call.abi_encode() != input.as_ref() {
            return Err("Noncanonical approval calldata".into());
        }
        review.spender = Some(call.spender.to_checksum(None));
        if token_info(chain, target).is_some() {
            review.title = "Approve ERC-20 allowance".into();
            review.allowance = Some(if call.value == U256::MAX {
                "Unlimited".into()
            } else {
                token_amount(chain, target, call.value)
            });
        } else if supports_erc721(provider, target).await? {
            review.title = "Approve NFT transfer".into();
            review
                .fields
                .push(("NFT token ID".into(), call.value.to_string()));
        } else {
            return Err("Approval for this token contract is unsupported".into());
        }
        return Ok(());
    }
    Err("This contract call cannot be displayed safely by WalletConnect".into())
}

pub(super) fn parse_transaction(
    value: &Value,
    address: Address,
    chain: u32,
) -> Result<TransactionRequest, String> {
    const FIELDS: &[&str] = &[
        "from",
        "to",
        "value",
        "data",
        "input",
        "chainId",
        "gas",
        "gasLimit",
        "gasPrice",
        "maxFeePerGas",
        "maxPriorityFeePerGas",
        "nonce",
        "type",
        "accessList",
    ];
    let mut fields = value
        .as_object()
        .ok_or("Expected a transaction object")?
        .clone();
    if fields.keys().any(|field| !FIELDS.contains(&field.as_str())) {
        return Err("The transaction uses unsupported fields".into());
    }
    if let Some(gas_limit) = fields.remove("gasLimit") {
        if fields.get("gas").is_some_and(|gas| gas != &gas_limit) {
            return Err("The transaction contains conflicting gas limits".into());
        }
        fields.insert("gas".into(), gas_limit);
    }
    let mut tx: TransactionRequest = serde_json::from_value(Value::Object(fields))
        .map_err(|_| "Invalid Ethereum transaction parameters")?;
    if tx.from != Some(address) || tx.chain_id.is_some_and(|id| id != u64::from(chain)) {
        return Err("Transaction sender or network differs from the approved session".into());
    }
    if !matches!(tx.to, Some(TxKind::Call(_))) || tx.transaction_type.is_some_and(|kind| kind != 2)
    {
        return Err("Only EIP-1559 transactions to an existing address are supported".into());
    }
    let input = tx
        .input
        .try_into_unique_input()
        .map_err(|_| "The transaction contains conflicting calldata")?;
    tx.input = TransactionInput::new(input.unwrap_or_default());
    tx.chain_id = Some(u64::from(chain));
    tx.transaction_type = Some(2);
    Ok(tx)
}

pub(super) async fn estimate_transaction(
    provider: &DynProvider,
    mut tx: TransactionRequest,
) -> Result<(TransactionRequest, U256), String> {
    let requested_gas = tx.gas.take();
    let legacy_fee_hint = tx.gas_price.take();
    let (gas, fees) = tokio::join!(
        provider.estimate_gas(tx.clone()),
        provider.estimate_eip1559_fees()
    );
    let gas = gas.map_err(
        |_| "RPC could not simulate the transaction. Check the balance and contract parameters.",
    )?;
    let fees = fees.map_err(|_| "Could not estimate EIP-1559 fees")?;
    let gas_limit = requested_gas.unwrap_or_else(|| gas.saturating_add(gas / 5));
    if gas_limit < gas {
        return Err("The requested gas limit is below the simulation estimate".into());
    }
    let max_fee = tx
        .max_fee_per_gas
        .or(legacy_fee_hint)
        .unwrap_or(fees.max_fee_per_gas);
    let priority_fee = tx
        .max_priority_fee_per_gas
        .unwrap_or(fees.max_priority_fee_per_gas.min(max_fee));
    if priority_fee > max_fee {
        return Err("The priority fee exceeds the maximum gas price".into());
    }
    tx.gas = Some(gas_limit);
    tx.max_fee_per_gas = Some(max_fee);
    tx.max_priority_fee_per_gas = Some(priority_fee);
    Ok((tx, U256::from(gas_limit) * U256::from(max_fee)))
}

#[cfg(test)]
mod tests {
    use alloy::{primitives::Bytes, providers::RootProvider, rpc::types::TransactionInput};

    use super::*;

    #[tokio::test]
    async fn walletconnect_reviews_actual_across_native_deposit_fields() {
        let origin = 11155111;
        let destination = 84532;
        let (spoke_pool, periphery) = across_contracts(origin);
        let depositor = address!("1000000000000000000000000000000000000000");
        let recipient = address!("2000000000000000000000000000000000000000");
        let amount = U256::from(100_000_000_000_000u64);
        let call = depositNativeCall {
            spokePool: spoke_pool,
            depositor,
            recipient: recipient.into_word(),
            inputToken: wrapped_eth(origin),
            inputAmount: amount,
            outputToken: wrapped_eth(destination).into_word(),
            outputAmount: U256::from(98_900_000_000_000u64),
            destinationChainId: U256::from(destination),
            exclusiveRelayer: B256::ZERO,
            quoteTimestamp: 0,
            fillDeadline: 3600,
            exclusivityParameter: 0,
            message: Bytes::new(),
        };
        let mut data = call.abi_encode();
        assert_eq!(data[..4], [0x60, 0x9e, 0xa0, 0x81]);
        data.extend([0x73, 0xc0, 0xde]);
        let mut tx = TransactionRequest::default()
            .to(periphery)
            .value(amount)
            .input(TransactionInput::new(data.into()));
        let provider = RootProvider::<alloy::network::Ethereum>::new_http(
            "http://127.0.0.1:1".parse().unwrap(),
        )
        .erased();
        let mut review = WalletConnectReview::default();
        review_transaction(&provider, &tx, &mut review, origin, depositor)
            .await
            .unwrap();
        assert_eq!(review.title, "Bridge via Across");
        assert!(
            review
                .fields
                .contains(&("Final recipient".into(), recipient.to_checksum(None)))
        );
        assert!(
            review
                .fields
                .contains(&("Destination".into(), "Base Sepolia (84532)".into()))
        );
        assert!(
            review
                .fields
                .iter()
                .any(|(label, value)| label == "Quoted output" && value.starts_with("0.0000989"))
        );

        tx.to = Some(alloy::primitives::TxKind::Call(recipient));
        assert!(
            review_transaction(&provider, &tx, &mut review, origin, depositor)
                .await
                .is_err()
        );
        tx.to = Some(alloy::primitives::TxKind::Call(periphery));
        tx.value = Some(U256::ONE);
        assert!(
            review_transaction(&provider, &tx, &mut review, origin, depositor)
                .await
                .is_err()
        );
    }
}
