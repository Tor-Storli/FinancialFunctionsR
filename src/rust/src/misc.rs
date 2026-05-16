// ── Miscellaneous financial functions ────────────────────────────────────────
// Excel: EFFECT, NOMINAL, DOLLARDE, DOLLARFR, FVSCHEDULE, RRI, PDURATION,
//        TBILLEQ, TBILLPRICE, TBILLYIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE, ODDLYIELD


use std::error::Error;
use crate::helpers::{ parse_date, year_frac, freq_per_year, add_months};
use crate::errors::{FinError, validate_basis, validate_frequency, validate_date_order, parse_f64_list};

/// Wrap a FinError into Box<dyn Error> for use with `?`.
#[inline]
fn fe(e: FinError) -> Box<dyn Error> { Box::new(e) }



// ── Odd period calculation helpers ────────────────────────────────────────────

fn basis_days(d1: chrono::NaiveDate, d2: chrono::NaiveDate, basis: i32) -> f64 {
    match basis {
        0 | 4 => year_frac(d1, d2, basis) * 360.0,
        _     => (d2 - d1).num_days() as f64,
    }
}

fn quasi_period(start: chrono::NaiveDate, end: chrono::NaiveDate, basis: i32, freq: i32) -> f64 {
    match basis {
        0 | 4 => 360.0 / freq as f64,
        _     => (end - start).num_days() as f64,
    }
}

fn count_coupon_periods(from: chrono::NaiveDate, to: chrono::NaiveDate, months: i32) -> i32 {
    let mut count = 0i32;
    let mut d = to;
    while d > from { count += 1; d = add_months(d, -months); }
    count
}

pub fn calc_oddfprice(
    settle: chrono::NaiveDate, mature: chrono::NaiveDate,
    issue: chrono::NaiveDate, first_coupon: chrono::NaiveDate,
    rate: f64, yld: f64, redemption: f64, freq: i32, basis: i32,
) -> f64 {
    let coupon  = rate * 100.0 / freq as f64;
    let yp      = yld / freq as f64;
    if (1.0 + yp).abs() < 1e-12 { return f64::NAN; }
    let v       = 1.0 / (1.0 + yp);
    let months  = 12 / freq;
    let qc_prev = add_months(first_coupon, -(months as i32));
    let e       = quasi_period(qc_prev, first_coupon, basis, freq);
    if e == 0.0 { return f64::NAN; }
    let dsc = basis_days(settle, first_coupon, basis);
    let w   = dsc / e;
    let n   = count_coupon_periods(first_coupon, mature, months as i32);
    let (odd_coupon, accrued) = if issue >= qc_prev {
        let dfc = basis_days(issue, first_coupon, basis);
        let dci = basis_days(issue, settle, basis);
        (coupon * dfc / e, coupon * dci / e)
    } else {
        let mut odd = 0.0f64;
        let mut acc = 0.0f64;
        let mut qc_end = first_coupon;
        loop {
            let qc_start     = add_months(qc_end, -(months as i32));
            let ei           = quasi_period(qc_start, qc_end, basis, freq);
            if ei == 0.0 { break; }
            let actual_start = if issue > qc_start { issue } else { qc_start };
            let dc           = basis_days(actual_start, qc_end, basis);
            odd += coupon * dc / ei;
            if settle >= actual_start && settle < qc_end {
                acc += coupon * basis_days(actual_start, settle, basis) / ei;
            } else if settle >= qc_end {
                acc += coupon * dc / ei;
            }
            if issue >= qc_start { break; }
            qc_end = qc_start;
        }
        (odd, acc)
    };
    let vw            = v.powf(w);
    let sum_regular: f64 = (1..=n).map(|k| v.powf(k as f64)).sum::<f64>() * coupon;
    let pv_redemption = redemption * v.powf(n as f64);
    vw * (odd_coupon + sum_regular + pv_redemption) - accrued
}

pub fn calc_oddfyield(
    settle: chrono::NaiveDate, mature: chrono::NaiveDate,
    issue: chrono::NaiveDate, first_coupon: chrono::NaiveDate,
    rate: f64, pr: f64, redemption: f64, freq: i32, basis: i32,
) -> f64 {
    let f = |y: f64| calc_oddfprice(settle, mature, issue, first_coupon,
                                     rate, y, redemption, freq, basis) - pr;
    bisect(f, 0.0, 1.0, 1e-9, 200)
}

pub fn calc_oddlprice(
    settle: chrono::NaiveDate, mature: chrono::NaiveDate,
    last_interest: chrono::NaiveDate, rate: f64, yld: f64,
    redemption: f64, freq: i32, basis: i32,
) -> f64 {
    let coupon = rate * 100.0 / freq as f64;
    let yp     = yld / freq as f64;
    if (1.0 + yp).abs() < 1e-12 { return f64::NAN; }
    let v      = 1.0 / (1.0 + yp);
    let months = 12 / freq;
    let qc_ref_end = add_months(last_interest, months as i32);
    let e          = quasi_period(last_interest, qc_ref_end, basis, freq);
    if e == 0.0 { return f64::NAN; }
    let dcl        = basis_days(last_interest, mature, basis);
    let nl         = dcl / e;
    let odd_coupon = coupon * nl;
    if settle >= last_interest {
        let dsm     = basis_days(settle, mature, basis);
        let w       = dsm / e;
        let dirty   = (odd_coupon + redemption) * v.powf(w);
        let dci     = basis_days(last_interest, settle, basis);
        let accrued = odd_coupon * dci / dcl;
        dirty - accrued
    } else {
        let mut qc_end = last_interest;
        loop {
            let prev = add_months(qc_end, -(months as i32));
            if prev <= settle { break; }
            qc_end = prev;
        }
        let qc_prev  = add_months(qc_end, -(months as i32));
        let e_settle = quasi_period(qc_prev, qc_end, basis, freq);
        if e_settle == 0.0 { return f64::NAN; }
        let dsc      = basis_days(settle, qc_end, basis);
        let w        = dsc / e_settle;
        let n_full   = count_coupon_periods(qc_end, last_interest, months as i32);
        let sum_reg: f64 = (0..=n_full).map(|k| coupon * v.powf(w + k as f64)).sum();
        let pv_final = (odd_coupon + redemption) * v.powf(w + n_full as f64 + nl);
        let dirty    = sum_reg + pv_final;
        let accrued  = coupon * (e_settle - dsc) / e_settle;
        dirty - accrued
    }
}

pub fn calc_oddlyield(
    settle: chrono::NaiveDate, mature: chrono::NaiveDate,
    last_interest: chrono::NaiveDate, rate: f64, pr: f64,
    redemption: f64, freq: i32, basis: i32,
) -> f64 {
    let f = |y: f64| calc_oddlprice(settle, mature, last_interest, rate, y, redemption, freq, basis) - pr;
    bisect(f, 0.0, 1.0, 1e-9, 100)
}

fn bisect<F: Fn(f64) -> f64>(f: F, mut lo: f64, mut hi: f64, tol: f64, max_iter: usize) -> f64 {
    for _ in 0..max_iter {
        let mid = (lo + hi) / 2.0;
        if (hi - lo) < tol { return mid; }
        if f(lo) * f(mid) < 0.0 { hi = mid; } else { lo = mid; }
    }
    (lo + hi) / 2.0
}

// ── Helper: parse two settlement/maturity date VARCHAR args ───────────────────
fn parse_two_dates_misc(
    func: &'static str, s: &str, m: &str,
) -> Result<(chrono::NaiveDate, chrono::NaiveDate), Box<dyn Error>> {
    let settle = parse_date(s).ok_or_else(|| fe(FinError::ParseDate {
        func, arg: "settlement", value: s.to_owned(),
    }))?;
    let mature = parse_date(m).ok_or_else(|| fe(FinError::ParseDate {
        func, arg: "maturity", value: m.to_owned(),
    }))?;
    validate_date_order(func, s, m, "settlement", "maturity", false).map_err(fe)?;
    Ok((settle, mature))
}

