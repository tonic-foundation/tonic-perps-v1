use crate::{Balance, BPS_DIVISOR, U256};

/// Assert that the predecessor account matches. Panic if not.
///
/// Usage:
/// ```ignore
/// pub fn set_interest_rate(&mut self, rate: u16) {
///     require_predecessor!(self.contract().owner_id);
///     // do stuff
/// }
#[macro_export]
macro_rules! require_predecessor {
    ($expected: expr) => {
        require_predecessor!($expected, "invalid caller");
    };
    ($expected: expr, $msg: expr) => {
        if env::predecessor_account_id() != $expected {
            env::panic_str($msg)
        }
    };
}

/// Implement a view function for a field on [VContract].
///
/// Usage:
/// ```ignore
/// impl VContract {
///     // impl_field_view!(method_name, field_name, return_type);
///     impl_field_view!(get_foo, foo, String);
/// }
/// ```
#[macro_export]
macro_rules! impl_field_view {
    ($name:ident, $field:ident, $t:ty) => {
        impl_field_view!($name, $field, $t, "");
    };

    ($name:ident, $field:ident, $t:ty, $doc:expr) => {
        #[doc = $doc]
        pub fn $name(&self) -> $t {
            self.contract().$field
        }
    };
}

/// Implement an admin-only setter function for a field on [VContract].
///
/// Usage:
/// ```ignore
/// impl VContract {
///     // impl_admin_field_setter!(method_name, field_name, field_type);
///     impl_admin_field_setter!(admin_set_foo, foo, String);
/// }
/// ```
#[macro_export]
macro_rules! impl_admin_field_setter {
    ($name:ident, $field:ident, $t:ty) => {
        impl_admin_field_setter!($name, $field, $t, "");
    };

    ($name:ident, $field:ident, $t:ty, $doc:expr) => {
        #[doc = $doc]
        pub fn $name(&mut self, $field: $t) -> $t {
            let mut contract = self.contract_mut();
            assert_caller!(contract.owner_id);
            contract.$field = $field;
        }
    };
}

#[macro_export]
macro_rules! module_tests {
    ($body: block) => {
        #[cfg(not(target_arch = "wasm32"))]
        #[cfg(test)]
        mod tests {
            use super::test_prelude::*;

            $body
        }
    };
}

#[macro_export]
macro_rules! BN {
    ($v:expr) => {
        BN(U256::from($v))
    };
}

pub struct BN(pub U256);

impl BN {
    pub fn mul(&self, v: u128) -> BN {
        BN(self.0 * U256::from(v))
    }

    pub fn div(&self, v: u128) -> BN {
        BN(self.0 / U256::from(v))
    }

    pub fn add(&self, v: u128) -> BN {
        BN(self.0 + U256::from(v))
    }

    pub fn sub(&self, v: u128) -> BN {
        BN(self.0 - U256::from(v))
    }

    pub fn as_u128(&self) -> u128 {
        self.0.as_u128()
    }

    pub fn as_u64(&self) -> u64 {
        self.0.as_u64()
    }

    pub fn add_bps(&self, bps: u16) -> BN {
        BN(self.0).mul(BPS_DIVISOR + bps as u128).div(BPS_DIVISOR)
    }

    pub fn sub_bps(&self, bps: u16) -> BN {
        BN(self.0).mul(BPS_DIVISOR - bps as u128).div(BPS_DIVISOR)
    }
}

pub fn round<T>(num: T, precision: T) -> T
where
    T: std::ops::Div<Output = T> + std::ops::Mul<Output = T> + std::marker::Copy,
{
    num / precision * precision
}

pub fn ratio<T, K, S>(a: T, num: K, denom: S) -> Balance
where
    T: Into<u128>,
    K: Into<u128>,
    S: Into<u128>,
{
    BN!(a.into()).mul(num.into()).div(denom.into()).as_u128()
}

pub mod u128_dec_format {
    use near_sdk::serde::de;
    use near_sdk::serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(num: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&num.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub fn convert_assets(
    amount_in: Balance,
    num_1: u128,
    num_2: u128,
    denom_1: u128,
    denom_2: u128,
) -> u128 {
    let num = BN!(num_1).mul(num_2).as_u128();
    let denom = BN!(denom_1).mul(denom_2).as_u128();
    ratio(amount_in, num, denom)
}

/// Generate setter and getter contract methods for a contract field.
/// Setter method is only callable by admins.
/// Optionally takes a third argument which will be the validator function.
/// Optionally takes a fourth argument which will be the methods base name.
///
/// # Example
///
/// ```ignore
/// contract_parameter!(test, String, |_contract, s| s.len() > 0, testing_var);
/// ```
///
/// Will generate the following methods:
///
/// ```ignore
/// #[near_bindgen]
/// impl VContract {
///     pub fn set_testing_var(&mut self, test: String) {
///         let contract = self.contract_mut();
///         contract.assert_admin();
///         let validator_function: fn(&Contract, &String) -> bool = |_contract, s| s.len() > 0;
///         let validator_result: bool = validator_function(contract, &test);
///         contract.test = test.into();
///     }
///     pub fn get_testing_var(&self) -> String {
///         let contract = self.contract();
///         contract.test.clone().into()
///     }
/// }
/// ```
#[macro_export]
macro_rules! contract_parameter {
    ($v:ident, $t:ty) => {
        contract_parameter!($v, $t, |_, _| true);
    };
    ($v:ident, $t:ty, $validator:expr) => {
        contract_parameter!($v, $t, $validator, $v);
    };
    ($v:ident, $t:ty, $validator:expr, $n:ident) => {
        paste::paste! {
            #[near_bindgen]
            impl VContract {
                pub fn [<set_ $n>](&mut self, $v: $t) {
                    let contract = self.contract_mut();
                    contract.assert_admin();
                    let validator_function: fn(&Contract, &$t) -> bool = $validator;
                    let validator_result: bool = validator_function(&contract, &$v);
                    assert!(validator_result);
                    contract.$v = $v.into();
                }
                pub fn [<get_ $n>](&self) -> $t {
                    let contract = self.contract();
                    contract.$v.clone().into()
                }
            }
        }
    };
}

/// Generate setter and getter contract methods for an asset field.
/// Setter method is only callable by admins.
/// Optionally takes a third argument which will be the validator function.
/// Optionally takes a fourth argument which will be the methods base name.
///
/// # Example
///
/// ```ignore
/// asset_parameter!(test, String, |_contract, s| s.len() > 0, testing_var);
/// ```
///
/// Will generate the following methods:
///
/// ```ignore
/// #[near_bindgen]
/// impl VContract {
///     pub fn set_testing_var(&mut self, asset_id: String, test: String) {
///         let contract = self.contract_mut();
///         contract.assert_admin();
///         let validator_function: fn(&Contract, &String) -> bool = |_contract, s| s.len() > 0;
///         let validator_result: bool = validator_function(contract, &test);
///         assert!((|s: &String| s.len() > 0)(test));
///         let mut asset = contract.assets.unwrap(&asset_id.clone().into());
///         asset.test = test.into();
///         contract.set_asset(&asset_id.into(), asset);
///     }
///     pub fn get_testing_var(&self, asset_id: String) -> String {
///         let contract = self.contract();
///         let asset = contract.assets.unwrap(&asset_id.into());
///         asset.test.clone().into()
///     }
/// }
/// ```
#[macro_export]
macro_rules! asset_parameter {
    ($v:ident, $t:ty) => {
        asset_parameter!($v, $t, |_, _| true);
    };
    ($v:ident, $t:ty, $validator:expr) => {
        asset_parameter!($v, $t, $validator, $v);
    };
    ($v:ident, $t:ty, $validator:expr, $n:ident) => {
        paste::paste! {
            #[near_bindgen]
            impl VContract {
                pub fn [<set_ $n>](&mut self, asset_id: String, $v: $t) {
                    let contract = self.contract_mut();
                    contract.assert_admin();
                    let validator_function: fn(&Contract, &$t) -> bool = $validator;
                    let validator_result: bool = validator_function(&contract, &$v);
                    assert!(validator_result);
                    let mut asset = contract.assets.unwrap(&asset_id.clone().into());
                    asset.$v = $v.into();
                    contract.set_asset(&asset_id.into(), asset);
                }
                pub fn [<get_ $n>](&self, asset_id: String) -> $t {
                    let contract = self.contract();
                    let asset = contract.assets.unwrap(&asset_id.into());
                    asset.$v.clone().into()
                }
            }
        }
    };
}
