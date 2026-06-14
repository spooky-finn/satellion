//! macOS Keychain storage for wallet passphrases. Private to the
//! `biometric` module.

use core_foundation::{
    base::{CFType, TCFType},
    boolean::CFBoolean,
    dictionary::CFDictionary,
    string::CFString,
};
use core_foundation_sys::base::{CFRelease, CFTypeRef};
use security_framework::passwords::{
    PasswordOptions, delete_generic_password_options, generic_password,
    set_generic_password_options,
};
use security_framework_sys::{
    base::errSecItemNotFound,
    item::{
        kSecAttrAccount, kSecAttrService, kSecClass, kSecClassGenericPassword, kSecReturnAttributes,
    },
    keychain_item::SecItemCopyMatching,
};

use super::BiometricError;

const SERVICE: &str = "com.satelliondao.satellion.wallet-passphrase";
const ACCOUNT_PREFIX: &str = "satellion_";
// Not re-exported by security-framework-sys; documented OSStatus code.
const ERR_SEC_INTERACTION_NOT_ALLOWED: i32 = -25308;

fn account_key(wallet_name: &str) -> String {
    format!("{}{}", ACCOUNT_PREFIX, wallet_name)
}

pub fn store(wallet_name: &str, passphrase: &[u8]) -> Result<(), BiometricError> {
    // Replace any prior entry so a re-enable always rotates the stored value.
    let _ = delete(wallet_name);
    let account = account_key(wallet_name);
    let mut opts = PasswordOptions::new_generic_password(SERVICE, &account);
    opts.set_label(&format!("Satellion wallet: {}", wallet_name));
    set_generic_password_options(passphrase, opts).map_err(|e| {
        BiometricError::Backend(format!("Failed to save passphrase to Keychain: {}", e))
    })
}

pub fn read(wallet_name: &str) -> Result<Vec<u8>, BiometricError> {
    let account = account_key(wallet_name);
    let opts = PasswordOptions::new_generic_password(SERVICE, &account);
    generic_password(opts).map_err(|e| match e.code() {
        c if c == errSecItemNotFound => BiometricError::NotConfigured,
        _ => BiometricError::Backend(format!("Failed to read passphrase from Keychain: {}", e)),
    })
}

pub fn delete(wallet_name: &str) -> Result<(), BiometricError> {
    let account = account_key(wallet_name);
    let opts = PasswordOptions::new_generic_password(SERVICE, &account);
    match delete_generic_password_options(opts) {
        Ok(()) => Ok(()),
        Err(e) if e.code() == errSecItemNotFound => Ok(()),
        Err(e) => Err(BiometricError::Backend(format!(
            "Failed to remove passphrase from Keychain: {}",
            e
        ))),
    }
}

pub fn has(wallet_name: &str) -> Result<bool, BiometricError> {
    // Probe attributes only — never triggers a biometric/Keychain prompt.
    let account = account_key(wallet_name);
    let pairs: Vec<(CFString, CFType)> = vec![
        (
            unsafe { CFString::wrap_under_get_rule(kSecClass) },
            unsafe { CFString::wrap_under_get_rule(kSecClassGenericPassword) }.as_CFType(),
        ),
        (
            unsafe { CFString::wrap_under_get_rule(kSecAttrService) },
            CFString::from(SERVICE).as_CFType(),
        ),
        (
            unsafe { CFString::wrap_under_get_rule(kSecAttrAccount) },
            CFString::from(account.as_str()).as_CFType(),
        ),
        (
            unsafe { CFString::wrap_under_get_rule(kSecReturnAttributes) },
            CFBoolean::from(true).as_CFType(),
        ),
    ];
    #[allow(deprecated)]
    let params = CFDictionary::from_CFType_pairs(&pairs);
    let mut ret: CFTypeRef = std::ptr::null();
    let status = unsafe { SecItemCopyMatching(params.as_concrete_TypeRef(), &mut ret) };
    if !ret.is_null() {
        unsafe { CFRelease(ret) };
    }
    match status {
        0 => Ok(true),
        s if s == errSecItemNotFound => Ok(false),
        s if s == ERR_SEC_INTERACTION_NOT_ALLOWED => Ok(true),
        other => Err(BiometricError::Backend(format!(
            "Keychain query failed: {}",
            other
        ))),
    }
}
