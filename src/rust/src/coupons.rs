// ── Coupon date functions ─────────────────────────────────────────────────────
// Excel: COUPDAYBS, COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM, COUPPCD


use chrono::NaiveDate;
use std::error::Error;
use crate::helpers::{ parse_date, add_months, year_frac, freq_per_year};
use crate::errors::{FinError, validate_basis, validate_frequency, validate_date_order};

/// Wrap a FinError into Box<dyn Error> for use with `?`.
#[inline]
fn fe(e: FinError) -> Box<dyn Error> { Box::new(e) }



// ── Coupon date arithmetic helpers ────────────────────────────────────────────

pub fn next_coupon(settlement: NaiveDate, maturity: NaiveDate, freq: i32) -> NaiveDate {
    let months_per_period = 12 / freq;
    let mut d = maturity;
    while d > settlement {
        let prev = add_months(d, -months_per_period);
        if prev <= settlement { return d; }
        d = prev;
    }
    add_months(d, months_per_period)
}

pub fn prev_coupon(settlement: NaiveDate, maturity: NaiveDate, freq: i32) -> NaiveDate {
    let ncd = next_coupon(settlement, maturity, freq);
    add_months(ncd, -(12 / freq))
}

pub fn coupon_period_days(settlement: NaiveDate, maturity: NaiveDate, freq: i32, basis: i32) -> f64 {
    let pcd = prev_coupon(settlement, maturity, freq);
    let ncd = next_coupon(settlement, maturity, freq);
    match basis {
        0 | 4 => 360.0 / freq as f64,
        _     => (ncd - pcd).num_days() as f64,
    }
}

pub fn calc_coupdaybs(settlement: NaiveDate, maturity: NaiveDate, freq: i32, basis: i32) -> f64 {
    let pcd = prev_coupon(settlement, maturity, freq);
    match basis {
        0 | 4 => year_frac(pcd, settlement, basis) * 360.0,
        _     => (settlement - pcd).num_days() as f64,
    }
}

pub fn calc_coupdays(settlement: NaiveDate, maturity: NaiveDate, freq: i32, basis: i32) -> f64 {
    coupon_period_days(settlement, maturity, freq, basis)
}

pub fn calc_coupdaysnc(settlement: NaiveDate, maturity: NaiveDate, freq: i32, basis: i32) -> f64 {
    let ncd = next_coupon(settlement, maturity, freq);
    match basis {
        0 | 4 => year_frac(settlement, ncd, basis) * 360.0,
        _     => (ncd - settlement).num_days() as f64,
    }
}

pub fn calc_coupncd(settlement: NaiveDate, maturity: NaiveDate, freq: i32) -> f64 {
    date_to_excel_serial(next_coupon(settlement, maturity, freq))
}

pub fn calc_couppcd(settlement: NaiveDate, maturity: NaiveDate, freq: i32) -> f64 {
    date_to_excel_serial(prev_coupon(settlement, maturity, freq))
}

pub fn calc_coupnum(settlement: NaiveDate, maturity: NaiveDate, freq: i32) -> f64 {
    let months_per_period = 12 / freq;
    let mut count = 0i32;
    let mut d = maturity;
    while d > settlement {
        count += 1;
        d = add_months(d, -months_per_period);
    }
    count as f64
}

fn date_to_excel_serial(d: NaiveDate) -> f64 {
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
    (d - epoch).num_days() as f64
}

// ── Shared input parsing ──────────────────────────────────────────────────────

/// Parse and validate all four inputs for basis-aware coupon functions.
fn parse_coupon_inputs(
    func: &'static str,
    settlement_str: &str,
    maturity_str: &str,
    freq_raw: i64,
    basis_raw: i64,
) -> Result<(NaiveDate, NaiveDate, i32, i32), FinError> {
    let settlement = parse_date(settlement_str).ok_or_else(|| FinError::ParseDate {
        func, arg: "settlement", value: settlement_str.to_owned(),
    })?;
    let maturity = parse_date(maturity_str).ok_or_else(|| FinError::ParseDate {
        func, arg: "maturity", value: maturity_str.to_owned(),
    })?;
    validate_date_order(func, settlement_str, maturity_str, "settlement", "maturity", false)?;
    validate_frequency(func, freq_raw)?;
    validate_basis(func, basis_raw)?;
    Ok((settlement, maturity, freq_per_year(freq_raw as i32), basis_raw as i32))
}

/// Parse and validate three inputs for the no-basis coupon functions.
fn parse_coupon_inputs_nobasis(
    func: &'static str,
    settlement_str: &str,
    maturity_str: &str,
    freq_raw: i64,
) -> Result<(NaiveDate, NaiveDate, i32), FinError> {
    let settlement = parse_date(settlement_str).ok_or_else(|| FinError::ParseDate {
        func, arg: "settlement", value: settlement_str.to_owned(),
    })?;
    let maturity = parse_date(maturity_str).ok_or_else(|| FinError::ParseDate {
        func, arg: "maturity", value: maturity_str.to_owned(),
    })?;
    validate_date_order(func, settlement_str, maturity_str, "settlement", "maturity", false)?;
    validate_frequency(func, freq_raw)?;
    Ok((settlement, maturity, freq_per_year(freq_raw as i32)))
}



