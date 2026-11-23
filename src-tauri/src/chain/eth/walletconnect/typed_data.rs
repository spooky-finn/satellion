use std::collections::{BTreeMap, BTreeSet};

use alloy::primitives::{B256, U256, hex};
use alloy_dyn_abi::eip712::TypedData;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::native_approval::visible_text;

use super::WalletConnectReview;

const MAX_SCHEMA_DEPTH: usize = 12;
const MAX_EXPANDED_FIELDS: usize = 256;
const MAX_REVIEW_BYTES: usize = 4608;

#[derive(Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Field {
    name: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Schema {
    types: BTreeMap<String, Vec<Field>>,
    primary_type: String,
    domain: BTreeMap<String, Value>,
    message: Value,
}

fn identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    name.len() <= 64
        && bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn atomic(kind: &str) -> bool {
    matches!(kind, "address" | "bool" | "string" | "bytes")
        || kind
            .strip_prefix("bytes")
            .and_then(|width| width.parse::<usize>().ok())
            .is_some_and(|width| (1..=32).contains(&width) && kind == format!("bytes{width}"))
        || ["uint", "int"].iter().any(|prefix| {
            kind.strip_prefix(prefix)
                .and_then(|width| width.parse::<usize>().ok())
                .is_some_and(|width| {
                    width > 0
                        && width <= 256
                        && width % 8 == 0
                        && kind == format!("{prefix}{width}")
                })
        })
}

fn array(kind: &str) -> Option<(&str, &str)> {
    let (element, length) = kind.strip_suffix(']')?.rsplit_once('[')?;
    Some((element, length))
}

fn schema_cost(
    kind: &str,
    schema: &Schema,
    stack: &mut Vec<String>,
    used: &mut BTreeSet<String>,
    remaining: &mut usize,
    depth: usize,
) -> Result<(), String> {
    if depth > MAX_SCHEMA_DEPTH || *remaining == 0 {
        return Err("EIP-712 types exceed the maximum complexity".into());
    }
    *remaining -= 1;
    if let Some((element, length)) = array(kind) {
        if !length.is_empty()
            && !length
                .parse::<usize>()
                .ok()
                .is_some_and(|n| (1..=64).contains(&n) && length == n.to_string())
        {
            return Err("Invalid or oversized EIP-712 array type".into());
        }
        return schema_cost(element, schema, stack, used, remaining, depth + 1);
    }
    if atomic(kind) {
        return Ok(());
    }
    if kind == "EIP712Domain" || !identifier(kind) || stack.iter().any(|name| name == kind) {
        return Err("Invalid or recursive EIP-712 type".into());
    }
    let fields = schema
        .types
        .get(kind)
        .ok_or("Missing EIP-712 type definition")?;
    used.insert(kind.into());
    stack.push(kind.into());
    let mut names = BTreeSet::new();
    for field in fields {
        if !identifier(&field.name) || !names.insert(&field.name) {
            return Err("Invalid or duplicate EIP-712 field".into());
        }
        schema_cost(&field.kind, schema, stack, used, remaining, depth + 1)?;
    }
    stack.pop();
    Ok(())
}

fn validate_value(
    value: &Value,
    kind: &str,
    schema: &Schema,
    budget: &mut usize,
) -> Result<(), String> {
    *budget = budget
        .checked_sub(1)
        .ok_or("EIP-712 message exceeds the maximum complexity")?;
    if let Some((element, length)) = array(kind) {
        let values = value.as_array().ok_or("Expected an EIP-712 array")?;
        if values.len() > 64 || (!length.is_empty() && length != values.len().to_string()) {
            return Err("Invalid or oversized EIP-712 array".into());
        }
        for value in values {
            validate_value(value, element, schema, budget)?;
        }
        return Ok(());
    }
    if atomic(kind) {
        if kind == "bool" && !value.is_boolean() {
            return Err("EIP-712 boolean values must be true or false".into());
        }
        if let Some(width) = kind.strip_prefix("bytes").filter(|width| !width.is_empty()) {
            let exact_bytes = width.parse::<usize>().expect("validated width");
            if !value
                .as_str()
                .and_then(|text| text.strip_prefix("0x"))
                .is_some_and(|data| {
                    data.len() == exact_bytes * 2 && data.bytes().all(|b| b.is_ascii_hexdigit())
                })
            {
                return Err("An EIP-712 fixed byte field has the wrong length".into());
            }
        }
        return Ok(());
    }
    let fields = schema
        .types
        .get(kind)
        .ok_or("Missing EIP-712 type definition")?;
    let object = value.as_object().ok_or("Expected an EIP-712 object")?;
    if object.len() != fields.len() {
        return Err("EIP-712 message fields do not match the signed schema".into());
    }
    for field in fields {
        let value = object
            .get(&field.name)
            .ok_or("Missing signed EIP-712 field")?;
        validate_value(value, &field.kind, schema, budget)?;
    }
    Ok(())
}

fn validate_domain(schema: &Schema) -> Result<(), String> {
    let mut expected = Vec::new();
    for (name, kind) in [
        ("name", "string"),
        ("version", "string"),
        ("chainId", "uint256"),
        ("verifyingContract", "address"),
        ("salt", "bytes32"),
    ] {
        if let Some(value) = schema.domain.get(name) {
            if value.is_null() {
                return Err("EIP-712 domain fields cannot be null".into());
            }
            expected.push(Field {
                name: name.into(),
                kind: kind.into(),
            });
        }
    }
    if expected.len() != schema.domain.len()
        || schema
            .types
            .get("EIP712Domain")
            .is_some_and(|given| *given != expected)
    {
        return Err("EIP-712 domain does not match the signed domain schema".into());
    }
    Ok(())
}

pub(super) fn review_typed_data(
    payload: &Value,
    chain: u32,
    review: &mut WalletConnectReview,
) -> Result<B256, String> {
    let payload = match payload.as_str() {
        Some(json) => {
            serde_json::from_str::<Value>(json).map_err(|_| "Invalid EIP-712 typed data")?
        }
        None => payload.clone(),
    };
    if !payload.is_object() {
        return Err("Expected an EIP-712 object".into());
    }
    let schema: Schema =
        serde_json::from_value(payload.clone()).map_err(|_| "Invalid EIP-712 schema")?;
    if schema.types.len() > 32
        || atomic(&schema.primary_type)
        || !schema.types.contains_key(&schema.primary_type)
    {
        return Err("Invalid or oversized EIP-712 schema".into());
    }
    validate_domain(&schema)?;
    let mut used = BTreeSet::new();
    let mut schema_budget = MAX_EXPANDED_FIELDS;
    schema_cost(
        &schema.primary_type,
        &schema,
        &mut Vec::new(),
        &mut used,
        &mut schema_budget,
        0,
    )?;
    let mut message_budget = MAX_EXPANDED_FIELDS;
    validate_value(
        &schema.message,
        &schema.primary_type,
        &schema,
        &mut message_budget,
    )?;

    let typed: TypedData =
        serde_json::from_value(payload).map_err(|_| "Invalid EIP-712 typed data")?;
    if typed.domain.chain_id != Some(U256::from(chain)) {
        return Err("EIP-712 domain must specify the approved network".into());
    }
    let primary = typed.primary_type.to_ascii_lowercase();
    let permit = primary.starts_with("permit") || primary.ends_with("authorization");
    if permit
        && typed
            .domain
            .verifying_contract
            .is_none_or(|address| address.is_zero())
    {
        return Err("An authorization must specify a verifying contract".into());
    }
    let encoded = typed
        .encode_type()
        .map_err(|_| "Could not resolve the signed EIP-712 schema")?;
    let message = serde_json::to_string_pretty(&json!({
        "domain": typed.domain, "message": typed.message
    }))
    .map_err(|_| "Could not display the signed EIP-712 data")?;
    let mut declarations = Vec::new();
    for kind in [schema.primary_type.clone()]
        .into_iter()
        .chain(used.into_iter().filter(|kind| *kind != schema.primary_type))
    {
        let fields = &schema.types[&kind];
        declarations.push(format!(
            "{kind}({})",
            fields
                .iter()
                .map(|field| format!("{} {}", field.kind, field.name))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    // The review must describe the same type encoding Alloy puts into the digest.
    if encoded != declarations.join("").replace(", ", ",") {
        return Err("The resolved EIP-712 schema differs from the reviewed schema".into());
    }
    let details = visible_text(&format!(
        "Signed schema\n{}\n\n{message}",
        declarations.join("\n")
    ));
    if details.len() > MAX_REVIEW_BYTES {
        return Err("EIP-712 data is too large to review safely".into());
    }
    review.title = if permit {
        "Authorize asset spending"
    } else {
        "Sign typed data"
    }
    .into();
    review
        .fields
        .push(("Primary type".into(), typed.primary_type.clone()));
    if let Some(contract) = typed.domain.verifying_contract {
        review.target = Some(contract.to_checksum(None));
    }
    for (field, label) in [
        ("spender", "Spender"),
        ("operator", "Operator"),
        ("deadline", "Deadline"),
    ] {
        if permit && let Some(value) = typed.message.get(field) {
            review
                .fields
                .push((label.into(), visible_text(&value.to_string())));
        }
    }
    review.details = Some(details);
    typed
        .eip712_signing_hash()
        .map_err(|_| "Could not hash the typed data".into())
}

pub(super) fn review_message(message: &[u8]) -> Result<String, String> {
    if message.len() > 1024 {
        return Err("Message is too large to review safely".into());
    }
    let text = std::str::from_utf8(message)
        .map_err(|_| "WalletConnect requires readable text for personal_sign")?;
    if text.trim().is_empty()
        || text
            .chars()
            .any(|ch| ch != '\n' && !matches!(ch, ' '..='~'))
    {
        return Err("WalletConnect requires printable ASCII for personal_sign".into());
    }
    Ok(format!(
        "Message supplied by the dApp\n{text}\n\nExact bytes: 0x{}",
        hex::encode(message)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn review() -> WalletConnectReview {
        WalletConnectReview {
            title: String::new(),
            target: None,
            value_eth: None,
            fee_eth: None,
            spender: None,
            allowance: None,
            details: None,
            switch_chain: None,
            fields: Vec::new(),
        }
    }

    #[test]
    fn walletconnect_eip712_review_matches_the_schema_and_chain() {
        let mut payload = json!({
            "types": {
                "EIP712Domain": [
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Permit": [{"name": "spender", "type": "address"}, {"name": "value", "type": "uint256"}]
            },
            "primaryType": "Permit",
            "domain": {"chainId": 1, "verifyingContract": "0x1000000000000000000000000000000000000000"},
            "message": {"spender": "0x2000000000000000000000000000000000000000", "value": "100"}
        });
        let mut displayed = review();
        let hash = review_typed_data(&payload, 1, &mut displayed).unwrap();
        let typed: TypedData = serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(hash, typed.eip712_signing_hash().unwrap());
        assert_eq!(displayed.title, "Authorize asset spending");
        assert!(
            displayed
                .details
                .unwrap()
                .contains("Permit(address spender, uint256 value)")
        );
        assert!(review_typed_data(&payload, 8453, &mut review()).is_err());
        payload["message"]["unsignedNote"] = json!("This field is not signed");
        assert!(review_typed_data(&payload, 1, &mut review()).is_err());
        payload["domain"].as_object_mut().unwrap().remove("chainId");
        assert!(review_typed_data(&payload, 1, &mut review()).is_err());
    }

    #[test]
    fn walletconnect_eip712_bounds_schema_expansion_before_hashing() {
        let mut types = serde_json::Map::new();
        for i in 0..9 {
            types.insert(
                format!("Node{i}"),
                json!([
                    {"name": "left", "type": format!("Node{}", i + 1)},
                    {"name": "right", "type": format!("Node{}", i + 1)}
                ]),
            );
        }
        types.insert(
            "Node9".into(),
            json!([{"name": "value", "type": "uint256"}]),
        );
        let data = json!({
            "types": types, "primaryType": "Node0",
            "domain": {"chainId": 1}, "message": {}
        });
        assert_eq!(
            review_typed_data(&data, 1, &mut review()).unwrap_err(),
            "EIP-712 types exceed the maximum complexity",
        );
    }
}
