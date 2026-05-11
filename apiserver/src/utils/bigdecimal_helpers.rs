use bigdecimal::BigDecimal;
use rust_decimal::Decimal;
use std::str::FromStr;

pub fn bd_to_dec(bd: BigDecimal) -> Decimal {
    Decimal::from_str(&bd.to_string()).unwrap_or_default()
}
