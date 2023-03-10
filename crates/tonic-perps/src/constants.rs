pub type DollarBalance = u128;

/// Internally, dollars have 6 decimals.
pub const DOLLAR_DECIMALS: u8 = 6;
pub const DOLLAR_DENOMINATION: DollarBalance = 10u128.pow(DOLLAR_DECIMALS as u32);
