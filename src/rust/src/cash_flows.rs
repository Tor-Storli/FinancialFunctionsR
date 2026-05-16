// ── Cash-flow functions ───────────────────────────────────────────────────────
// Excel: NPV, IRR, MIRR, XNPV, XIRR
//
// NOTE: The `financial` crate is NOT used here. Its IRR/MIRR/XIRR implementations
// panic with "cannot unwind" on certain inputs, which bypasses catch_unwind and
// crashes DuckDB. All five functions are implemented from scratch.


use std::error::Error;
use crate::helpers::{ parse_csv_dates};
use crate::errors::{FinError, parse_f64_list, validate_list_lengths, validate_non_empty, validate_rate};

/// Wrap a FinError into Box<dyn Error> for use with `?`.
#[inline]
fn fe(e: FinError) -> Box<dyn Error> { Box::new(e) }



// ── Pure-Rust implementations ─────────────────────────────────────────────────

pub fn npv_calc(rate: f64, values: &[f64]) -> f64 {
    let base = 1.0 + rate;
    if base.abs() < 1e-12 { return f64::INFINITY; }
    values.iter().enumerate()
        .map(|(t, &v)| v / base.powi((t + 1) as i32))
        .sum()
}

fn npv_at_zero(values: &[f64], rate: f64) -> f64 {
    let base = 1.0 + rate;
    if base.abs() < 1e-12 { return f64::INFINITY; }
    values.iter().enumerate()
        .map(|(t, &v)| v / base.powi(t as i32))
        .sum()
}

pub fn calc_irr(values: &[f64]) -> f64 {
    if values.len() < 2 { return f64::NAN; }
    if !values.iter().any(|&v| v > 0.0) || !values.iter().any(|&v| v < 0.0) {
        return f64::NAN;
    }
    let points: &[f64] = &[-0.9999, -0.5, -0.2, -0.1, -0.01, 0.0,
                             0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 100.0];
    for w in points.windows(2) {
        let (lo, hi) = (w[0], w[1]);
        let flo = npv_at_zero(values, lo);
        let fhi = npv_at_zero(values, hi);
        if flo.is_finite() && fhi.is_finite() && flo * fhi <= 0.0 {
            return bisect(|r| npv_at_zero(values, r), lo, hi, 1e-10, 200);
        }
    }
    f64::NAN
}

pub fn calc_mirr(values: &[f64], finance_rate: f64, reinvest_rate: f64) -> f64 {
    let n = values.len();
    if n < 2 { return f64::NAN; }
    if !values.iter().any(|&v| v > 0.0) || !values.iter().any(|&v| v < 0.0) {
        return f64::NAN;
    }
    let mut numer = 0.0f64;
    let mut denom = 0.0f64;
    for (t, &v) in values.iter().enumerate() {
        if v > 0.0 {
            numer += v / (1.0 + reinvest_rate).powi(t as i32);
        } else if v < 0.0 {
            denom += v / (1.0 + finance_rate).powi(t as i32);
        }
    }
    if denom.abs() < 1e-12 { return f64::NAN; }
    (numer / denom.abs()).powf(1.0 / (n - 1) as f64) * (1.0 + reinvest_rate) - 1.0
}

pub fn xnpv_at(values: &[f64], year_fracs: &[f64], rate: f64) -> f64 {
    let base = 1.0 + rate;
    if base.abs() < 1e-12 { return f64::INFINITY; }
    values.iter().zip(year_fracs.iter())
        .map(|(&v, &t)| v / base.powf(t))
        .sum()
}

pub fn calc_xirr(values: &[f64], dates: &[chrono::NaiveDate]) -> f64 {
    if values.len() < 2 || values.len() != dates.len() { return f64::NAN; }
    if !values.iter().any(|&v| v > 0.0) || !values.iter().any(|&v| v < 0.0) {
        return f64::NAN;
    }
    let d0 = dates[0];
    let year_fracs: Vec<f64> = dates.iter()
        .map(|&d| (d - d0).num_days() as f64 / 365.0)
        .collect();
    let points: &[f64] = &[-0.9999, -0.5, -0.2, -0.1, -0.01, 0.0,
                             0.01, 0.1, 0.5, 1.0, 5.0, 10.0];
    for w in points.windows(2) {
        let (lo, hi) = (w[0], w[1]);
        let flo = xnpv_at(values, &year_fracs, lo);
        let fhi = xnpv_at(values, &year_fracs, hi);
        if flo.is_finite() && fhi.is_finite() && flo * fhi <= 0.0 {
            return bisect(|r| xnpv_at(values, &year_fracs, r), lo, hi, 1e-10, 200);
        }
    }
    f64::NAN
}

fn bisect<F: Fn(f64) -> f64>(f: F, mut lo: f64, mut hi: f64, tol: f64, max_iter: usize) -> f64 {
    for _ in 0..max_iter {
        let mid = (lo + hi) / 2.0;
        if (hi - lo).abs() < tol { return mid; }
        let fm = f(mid);
        if fm.abs() < 1e-12 { return mid; }
        if f(lo) * fm <= 0.0 { hi = mid; } else { lo = mid; }
    }
    (lo + hi) / 2.0
}

