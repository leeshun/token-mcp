use ethers::prelude::*;

pub fn compare_float_str(a: &str, b: &str) -> Option<std::cmp::Ordering> {
    let a_f: f64 = a.parse().ok()?;
    let b_f: f64 = b.parse().ok()?;
    a_f.partial_cmp(&b_f)
}

pub fn get_amount_out_v2(amount_in: U256, reserve_in: U256, reserve_out: U256) -> U256 {
    assert!(amount_in > U256::zero());
    assert!(reserve_in > U256::zero());
    assert!(reserve_out > U256::zero());

    let fee_basis = U256::from(997_u64);
    let amount_in_with_fee = amount_in * fee_basis;
    let numerator = amount_in_with_fee * reserve_out;
    let denominator = reserve_in * U256::from(1000_u64) + amount_in_with_fee;
    numerator / denominator
}
