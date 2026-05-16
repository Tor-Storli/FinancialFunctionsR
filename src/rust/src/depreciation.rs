// ── Depreciation functions ────────────────────────────────────────────────────
// Excel: SLN, SYD, DB, DDB, VDB, AMORDEGRC, AMORLINC


use std::error::Error;
use crate::errors::{FinError, validate_cost, validate_life, validate_period, validate_salvage};

/// Wrap a FinError into Box<dyn Error> for use with `?`.
#[inline]
fn fe(e: FinError) -> Box<dyn Error> { Box::new(e) }


// ── Pure-Rust helpers ─────────────────────────────────────────────────────────

pub fn calc_sln(cost: f64, salvage: f64, life: f64) -> f64 {
    if life == 0.0 { return f64::NAN; }
    (cost - salvage) / life
}

pub fn calc_syd(cost: f64, salvage: f64, life: f64, per: f64) -> f64 {
    if life == 0.0 { return f64::NAN; }
    let sum_digits = life * (life + 1.0) / 2.0;
    (cost - salvage) * (life - per + 1.0) / sum_digits
}

pub fn calc_db(cost: f64, salvage: f64, life: f64, per: f64, month: f64) -> f64 {
    if cost == 0.0 { return 0.0; }
    if life == 0.0 { return f64::NAN; }
    let rate = ((1.0 - (salvage / cost).powf(1.0 / life)) * 1000.0).round() / 1000.0;
    let mut book = cost;
    let mut dep = 0.0;
    for p in 1..=(per as usize) {
        dep = if p == 1 {
            cost * rate * month / 12.0
        } else if p == (life as usize + 1) {
            (book - salvage.max(0.0)) * rate * (12.0 - month) / 12.0
        } else {
            book * rate
        };
        book -= dep;
    }
    dep
}

pub fn calc_ddb(cost: f64, salvage: f64, life: f64, per: f64, factor: f64) -> f64 {
    if life == 0.0 { return f64::NAN; }
    let rate = factor / life;
    let mut book = cost;
    let mut dep = 0.0;
    for _p in 1..=(per as usize) {
        dep = (book * rate).min(book - salvage);
        if dep < 0.0 { dep = 0.0; }
        book -= dep;
    }
    dep
}

pub fn calc_vdb(cost: f64, salvage: f64, life: f64, start_per: f64, end_per: f64, factor: f64, no_switch: bool) -> f64 {
    if life == 0.0 { return f64::NAN; }
    let rate = factor / life;
    let mut total = 0.0;
    let i_start = start_per.floor() as usize;
    let i_end   = end_per.ceil() as usize;
    let n = life.ceil() as usize;
    let mut book = cost;
    let mut schedule = vec![0.0f64; n + 1];
    for p in 1..=n {
        let ddb = (book * rate).max(0.0);
        let sl  = if life - (p as f64 - 1.0) > 0.0 {
            (book - salvage) / (life - (p as f64 - 1.0))
        } else { 0.0 };
        let dep = if !no_switch && sl > ddb { sl } else { ddb };
        let dep = dep.min(book - salvage).max(0.0);
        schedule[p] = dep;
        book -= dep;
    }
    for p in (i_start + 1)..=i_end {
        let frac = if p as f64 <= start_per {
            0.0
        } else if (p as f64 - 1.0) < start_per {
            (p as f64 - start_per).min(1.0)
        } else if p as f64 > end_per {
            end_per - (p as f64 - 1.0)
        } else {
            1.0
        };
        if p <= n { total += schedule[p] * frac; }
    }
    total
}

pub fn calc_amorlinc(cost: f64, _date_purch: f64, _first_period: f64, salvage: f64, _period: f64, rate: f64, _basis: f64) -> f64 {
    let annual_dep = cost * rate;
    let total_dep  = cost - salvage;
    annual_dep.min(total_dep)
}

pub fn calc_amordegrc(cost: f64, date_purch: f64, first_period: f64, salvage: f64, period: f64, rate: f64, basis: f64) -> f64 {
    if rate <= 0.0 || cost <= salvage { return f64::NAN; }
    let life  = 1.0 / rate;
    let coeff = if life >= 3.0 && life < 4.0      { 1.5 }
                else if life >= 5.0 && life < 6.0 { 2.0 }
                else if life >= 6.0                { 2.5 }
                else                               { return f64::NAN; };
    let depr_rate = rate * coeff;
    let year_days = match basis as i32 {
        3     => 365.0,
        0 | 4 => 360.0,
        2     => 360.0,
        _     => 365.0,
    };
    let first_frac = ((first_period - date_purch) / year_days).clamp(0.0, 1.0);
    let dep0 = (cost * depr_rate * first_frac).floor().min(cost - salvage).max(0.0);
    if period == 0.0 { return dep0; }
    let mut book = cost - dep0;
    let total_periods = life.ceil() as usize;
    for p in 1..=(period as usize) {
        if book <= salvage { return 0.0; }
        let remaining = total_periods.saturating_sub(p);
        let dep = if remaining == 0 {
            (book - salvage).max(0.0)
        } else if remaining == 1 {
            (book * 0.5).floor().min(book - salvage).max(0.0)
        } else {
            (book * depr_rate).floor().min(book - salvage).max(0.0)
        };
        if p == period as usize { return dep; }
        book -= dep;
    }
    0.0
}

