// ── Bond/security pricing functions ──────────────────────────────────────────
// Excel: PRICE, PRICEDISC, PRICEMAT, YIELD, YIELDDISC, YIELDMAT,
//        DISC, INTRATE, RECEIVED, DURATION, MDURATION, ACCRINT, ACCRINTM


use std::error::Error;
use crate::helpers::{ parse_date, year_frac, freq_per_year};
use crate::coupons::{next_coupon, calc_coupdays, calc_coupnum};
use crate::errors::{FinError, validate_basis, validate_frequency, validate_date_order};

/// Wrap a FinError into Box<dyn Error> for use with `?`.
#[inline]
fn fe(e: FinError) -> Box<dyn Error> { Box::new(e) }


// ── Pure-Rust helpers ─────────────────────────────────────────────────────────

pub fn calc_price(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    rate: f64, yld: f64, redemption: f64, freq: i32, basis: i32) -> f64
{
    let n = calc_coupnum(settle, maturity, freq);
    if n <= 0.0 { return f64::NAN; }
    bond_price(settle, maturity, rate, yld, redemption, freq, basis)
}

fn bond_price(
    settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    rate: f64, yld: f64, redemption: f64, freq: i32, basis: i32,
) -> f64 {
    let n = calc_coupnum(settle, maturity, freq) as i32;
    if n <= 0 { return f64::NAN; }
    let coupon  = rate * 100.0 / freq as f64;
    let yld_per = yld / freq as f64;
    let e = calc_coupdays(settle, maturity, freq, basis);
    if e == 0.0 { return f64::NAN; }
    let dsc = match basis {
        0 | 4 => {
            let ncd = next_coupon(settle, maturity, freq);
            year_frac(settle, ncd, basis) * 360.0
        }
        _ => (next_coupon(settle, maturity, freq) - settle).num_days() as f64,
    };
    let w = dsc / e;
    let dirty = if yld_per.abs() < 1e-12 {
        coupon * n as f64 + redemption
    } else {
        let v  = 1.0 / (1.0 + yld_per);
        let vw = v.powf(w);
        let mut pv = 0.0f64;
        for k in 0..n { pv += coupon * v.powf(k as f64); }
        pv += redemption * v.powf((n - 1) as f64);
        pv * vw
    };
    let accrued = coupon * (e - dsc) / e;
    dirty - accrued
}

pub fn calc_pricedisc(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    discount: f64, redemption: f64, basis: i32) -> f64
{
    let yf = year_frac(settle, maturity, basis);
    redemption * (1.0 - discount * yf)
}

pub fn calc_pricemat(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    issue: chrono::NaiveDate, rate: f64, yld: f64, basis: i32) -> f64
{
    let dim = year_frac(issue, maturity, basis);
    let dis = year_frac(issue, settle, basis);
    let dsm = year_frac(settle, maturity, basis);
    if (1.0 + yld * dsm).abs() < 1e-12 { return f64::NAN; }
    (100.0 * (1.0 + rate * dim)) / (1.0 + yld * dsm) - 100.0 * rate * dis
}

pub fn calc_yield(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    rate: f64, pr: f64, redemption: f64, freq: i32, basis: i32) -> f64
{
    let f = |y: f64| bond_price(settle, maturity, rate, y, redemption, freq, basis) - pr;
    newton_raphson(f, 0.05, 1e-10, 200)
}

pub fn calc_yielddisc(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    pr: f64, redemption: f64, basis: i32) -> f64
{
    let yf = year_frac(settle, maturity, basis);
    if yf == 0.0 || pr == 0.0 { return f64::NAN; }
    (redemption / pr - 1.0) / yf
}

pub fn calc_yieldmat(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    issue: chrono::NaiveDate, rate: f64, pr: f64, basis: i32) -> f64
{
    let dim = year_frac(issue, maturity, basis);
    let dis = year_frac(issue, settle, basis);
    let dsm = year_frac(settle, maturity, basis);
    if dsm.abs() < 1e-12 { return f64::NAN; }
    let numer = 100.0 * (1.0 + rate * dim);
    let denom = pr + 100.0 * rate * dis;
    if denom.abs() < 1e-12 { return f64::NAN; }
    (numer / denom - 1.0) / dsm
}

pub fn calc_disc(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    pr: f64, redemption: f64, basis: i32) -> f64
{
    let yf = year_frac(settle, maturity, basis);
    if redemption == 0.0 || yf == 0.0 { return f64::NAN; }
    (redemption - pr) / redemption / yf
}

pub fn calc_intrate(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    investment: f64, redemption: f64, basis: i32) -> f64
{
    let yf = year_frac(settle, maturity, basis);
    if investment == 0.0 || yf == 0.0 { return f64::NAN; }
    (redemption / investment - 1.0) / yf
}

pub fn calc_received(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    investment: f64, discount: f64, basis: i32) -> f64
{
    let yf    = year_frac(settle, maturity, basis);
    let denom = 1.0 - discount * yf;
    if denom.abs() < 1e-12 { return f64::NAN; }
    investment / denom
}

pub fn calc_duration(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    coupon: f64, yld: f64, freq: i32, basis: i32) -> f64
{
    let n = calc_coupnum(settle, maturity, freq) as i32;
    if n <= 0 { return f64::NAN; }
    let c   = coupon * 100.0 / freq as f64;
    let y   = yld / freq as f64;
    let dsc = (next_coupon(settle, maturity, freq) - settle).num_days() as f64;
    let e   = calc_coupdays(settle, maturity, freq, basis);
    if e == 0.0 { return f64::NAN; }
    let w = dsc / e;
    let v = 1.0 / (1.0 + y);
    let mut sum_pv = 0.0f64;
    let mut sum_t_pv = 0.0f64;
    for k in 0..n {
        let t  = w + k as f64;
        let pv = c * v.powf(t);
        sum_pv   += pv;
        sum_t_pv += t * pv;
    }
    let t_red  = w + (n - 1) as f64;
    let pv_red = 100.0 * v.powf(t_red);
    sum_pv   += pv_red;
    sum_t_pv += t_red * pv_red;
    if sum_pv.abs() < 1e-12 { return f64::NAN; }
    (sum_t_pv / sum_pv) / freq as f64
}

pub fn calc_mduration(settle: chrono::NaiveDate, maturity: chrono::NaiveDate,
    coupon: f64, yld: f64, freq: i32, basis: i32) -> f64
{
    let dur   = calc_duration(settle, maturity, coupon, yld, freq, basis);
    let denom = 1.0 + yld / freq as f64;
    if denom.abs() < 1e-12 { return f64::NAN; }
    dur / denom
}

pub fn calc_accrint(issue: chrono::NaiveDate, _first_interest: chrono::NaiveDate,
    settle: chrono::NaiveDate, rate: f64, par: f64, freq: i32, basis: i32) -> f64
{
    let nc = calc_coupnum(settle, settle, freq).max(1.0);
    let _ = (nc, freq);
    par * rate * year_frac(issue, settle, basis)
}

pub fn calc_accrintm(issue: chrono::NaiveDate, settle: chrono::NaiveDate,
    rate: f64, par: f64, basis: i32) -> f64
{
    par * rate * year_frac(issue, settle, basis)
}

fn newton_raphson<F: Fn(f64) -> f64>(f: F, guess: f64, tol: f64, max_iter: usize) -> f64 {
    let mut x = guess;
    for _ in 0..max_iter {
        let fx  = f(x);
        let dfx = (f(x + 1e-7) - fx) / 1e-7;
        if dfx.abs() < 1e-20 { return f64::NAN; }
        let x_new = x - fx / dfx;
        if (x_new - x).abs() < tol { return x_new; }
        x = x_new;
    }
    f64::NAN
}

// ── Helper: parse two dates with order and code validation ────────────────────

fn parse_two_dates(
    func: &'static str,
    s: &str, m: &str,
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

fn parse_three_dates(
    func: &'static str,
    s: &str, m: &str, extra: &str,
    extra_arg: &'static str,
) -> Result<(chrono::NaiveDate, chrono::NaiveDate, chrono::NaiveDate), Box<dyn Error>> {
    let settle = parse_date(s).ok_or_else(|| fe(FinError::ParseDate {
        func, arg: "settlement", value: s.to_owned(),
    }))?;
    let mature = parse_date(m).ok_or_else(|| fe(FinError::ParseDate {
        func, arg: "maturity", value: m.to_owned(),
    }))?;
    let ex = parse_date(extra).ok_or_else(|| fe(FinError::ParseDate {
        func, arg: extra_arg, value: extra.to_owned(),
    }))?;
    validate_date_order(func, s, m, "settlement", "maturity", false).map_err(fe)?;
    Ok((settle, mature, ex))
}

