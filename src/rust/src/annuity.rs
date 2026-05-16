// ── Annuity functions ─────────────────────────────────────────────────────────
// Excel: FV, PV, PMT, IPMT, PPMT, CUMIPMT, CUMPRINC, NPER, RATE, ISPMT


use std::error::Error;
use crate::errors::{FinError, validate_nper, validate_per, validate_rate};

/// Wrap a FinError into Box<dyn Error> for use with `?`.
#[inline]
fn fe(e: FinError) -> Box<dyn Error> { Box::new(e) }

// ── Pure-Rust calculation helpers ────────────────────────────────────────────

pub fn calc_pmt(rate: f64, nper: f64, pv: f64, fv: f64, pmt_at_beg: bool) -> f64 {
    if nper == 0.0 { return f64::NAN; }
    if rate == 0.0 { return -(pv + fv) / nper; }
    let r1n = (1.0 + rate).powf(nper);
    let due = if pmt_at_beg { 1.0 + rate } else { 1.0 };
    let denom = (r1n - 1.0) / rate * due;
    if denom.abs() < 1e-12 { return f64::NAN; }
    -(pv * r1n + fv) / denom
}

pub fn calc_ipmt(rate: f64, per: f64, nper: f64, pv: f64, fv: f64, pmt_at_beg: bool) -> f64 {
    if rate == 0.0 { return 0.0; }
    let pmt = calc_pmt(rate, nper, pv, fv, pmt_at_beg);
    let r1  = 1.0 + rate;
    let balance = pv * r1.powf(per - 1.0)
        + pmt * (r1.powf(per - 1.0) - 1.0) / rate;
    -(balance * rate)
}

pub fn calc_ppmt(rate: f64, per: f64, nper: f64, pv: f64, fv: f64, pmt_at_beg: bool) -> f64 {
    calc_pmt(rate, nper, pv, fv, pmt_at_beg) - calc_ipmt(rate, per, nper, pv, fv, pmt_at_beg)
}

pub fn calc_cumipmt(rate: f64, nper: f64, pv: f64, start: f64, end: f64, pmt_at_beg: bool) -> f64 {
    (start as usize..=end as usize)
        .map(|p| calc_ipmt(rate, p as f64, nper, pv, 0.0, pmt_at_beg))
        .sum()
}

pub fn calc_cumprinc(rate: f64, nper: f64, pv: f64, start: f64, end: f64, pmt_at_beg: bool) -> f64 {
    (start as usize..=end as usize)
        .map(|p| calc_ppmt(rate, p as f64, nper, pv, 0.0, pmt_at_beg))
        .sum()
}

pub fn calc_nper(rate: f64, pmt: f64, pv: f64, fv: f64, pmt_at_beg: bool) -> f64 {
    if rate == 0.0 {
        if pmt.abs() < 1e-12 { return f64::NAN; }
        return -(pv + fv) / pmt;
    }
    let due = if pmt_at_beg { 1.0 + rate } else { 1.0 };
    let adjusted_pmt = pmt * due;
    let num = adjusted_pmt - fv * rate;
    let den = adjusted_pmt + pv * rate;
    if den.abs() < 1e-12 || num / den <= 0.0 { return f64::NAN; }
    (num / den).ln() / (1.0 + rate).ln()
}

pub fn calc_rate(nper: f64, pmt: f64, pv: f64, fv: f64, pmt_at_beg: bool, guess: f64) -> f64 {
    let max_iter = 300;
    let tol = 1e-10;
    let mut r = guess;
    for _ in 0..max_iter {
        let r1  = 1.0 + r;
        let r1n = r1.powf(nper);
        let due = if pmt_at_beg { r1 } else { 1.0 };
        let f = if r.abs() < 1e-12 {
            pv + pmt * nper * due + fv
        } else {
            pv * r1n + pmt * due * (r1n - 1.0) / r + fv
        };
        let df = if r.abs() < 1e-12 {
            pv * nper + pmt * (nper * due / r1 + (r1n - 1.0) / r)
        } else {
            pv * nper * r1.powf(nper - 1.0)
                + pmt * due * (nper * r1.powf(nper - 1.0) * r - (r1n - 1.0)) / (r * r)
        };
        if df.abs() < 1e-20 { return f64::NAN; }
        let r_new = r - f / df;
        if (r_new - r).abs() < tol { return r_new; }
        r = r_new;
    }
    f64::NAN
}

pub fn calc_ispmt(rate: f64, per: f64, nper: f64, pv: f64) -> f64 {
    if nper == 0.0 { return f64::NAN; }
    let principal_per_period = pv / nper;
    let remaining = pv - principal_per_period * (per - 1.0);
    -(remaining * rate)
}



