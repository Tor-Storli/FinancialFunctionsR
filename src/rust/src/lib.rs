// ── FinancialFunctions R package — extendr entry point ────────────────────────
//
// This file contains ONLY the thin #[extendr] wrappers that expose each
// calc_* function to R. All calculation logic lives in the same module files
// copied from the DuckDB extension (annuity.rs, cash_flows.rs, etc.) — those
// files are unchanged except that the `use duckdb::...` imports and the
// VScalar structs are removed, leaving only the pure `pub fn calc_*` functions.
#![allow(unused_imports, dead_code)]

use extendr_api::prelude::*;
use chrono::NaiveDate;

mod helpers;
mod errors;      // ← add this
mod annuity;
mod cash_flows;
mod depreciation;
mod coupons;
mod bonds;
mod misc;

use crate::cash_flows::{npv_calc, calc_irr, calc_mirr, xnpv_at, calc_xirr};
use annuity::*;
use depreciation::*;
use coupons::*;
use bonds::*;
use misc::*;

// ── Helper: parse a date string, returning None on failure ────────────────────
fn parse_date_str(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()
}

// ── Helper: NaN → R NA convention ────────────────────────────────────────────
// R represents missing numerics as NA (which is f64::NAN internally).
// All calc_* functions already return f64::NAN on bad input, so no conversion
// is needed — R will display them as NA automatically.


// ══════════════════════════════════════════════════════════════════════════════
// ANNUITY FUNCTIONS
// ══════════════════════════════════════════════════════════════════════════════

/// Compute the Future Value of an investment.
///
/// Equivalent to Excel's FV function.
///
/// @param rate Interest rate per period (numeric).
/// @param nper Total number of payment periods (numeric).
/// @param pmt Payment made each period; negative = cash outflow (numeric).
/// @param pv Present value / initial investment (numeric, default 0).
/// @param pmt_at_beginning TRUE if payments are due at the beginning of each
///   period (annuity-due); FALSE for end-of-period (annuity-immediate).
/// @return Future value as a double. Returns NA on invalid input.
/// @examples
/// # Excel: FV(0.06/12, 10, -200, -500, TRUE) = 2581.40
/// fv(0.06/12, 10, -200, -500, TRUE)
/// @export
#[extendr]
fn fv(rate: f64, nper: f64, pmt: f64, pv: f64, pmt_at_beginning: bool) -> f64 {
    financial::fv(rate, nper, Some(pmt), Some(pv), Some(pmt_at_beginning))
}

/// Compute the Present Value of an investment.
///
/// Equivalent to Excel's PV function.
///
/// @param rate Interest rate per period (numeric).
/// @param nper Total number of payment periods (numeric).
/// @param pmt Payment made each period (numeric).
/// @param fv Future value remaining after the last payment (numeric, default 0).
/// @param pmt_at_beginning TRUE if payments are due at the beginning of each period.
/// @return Present value as a double. Returns NA on invalid input.
/// @examples
/// # Excel: PV(0.08/12, 20*12, 500, 0, FALSE) = -59777.15
/// pv(0.08/12, 240, 500, 0, FALSE)
/// @export
#[extendr]
fn pv(rate: f64, nper: f64, pmt: f64, fv: f64, pmt_at_beginning: bool) -> f64 {
    financial::pv(rate, nper, Some(pmt), Some(fv), Some(pmt_at_beginning))
}

/// Compute the periodic Payment for a loan or annuity.
///
/// Equivalent to Excel's PMT function.
///
/// @param rate Interest rate per period (numeric).
/// @param nper Total number of payment periods (numeric).
/// @param pv Present value (loan amount) (numeric).
/// @param fv Future value after last payment (numeric, default 0).
/// @param pmt_at_beginning TRUE if payments are due at the beginning of each period.
/// @return Periodic payment as a double (negative = cash outflow). Returns NA on invalid input.
/// @examples
/// # Excel: PMT(0.08/12, 10, 10000, 0, FALSE) = -1037.03
/// pmt(0.08/12, 10, 10000, 0, FALSE)
/// @export
#[extendr]
fn pmt(rate: f64, nper: f64, pv: f64, fv: f64, pmt_at_beginning: bool) -> f64 {
    calc_pmt(rate, nper, pv, fv, pmt_at_beginning)
}

/// Compute the Interest portion of a loan payment for a given period.
///
/// Equivalent to Excel's IPMT function.
///
/// @param rate Interest rate per period (numeric).
/// @param per The period for which to compute interest; must be in [1, nper] (numeric).
/// @param nper Total number of payment periods (numeric).
/// @param pv Present value (numeric).
/// @param fv Future value (numeric, default 0).
/// @param pmt_at_beginning TRUE if payments are due at the beginning of each period.
/// @return Interest payment for the given period. Returns NA if per is outside [1, nper].
/// @examples
/// # Excel: IPMT(0.10/12, 1, 3, 8000, 0, FALSE) = -66.67
/// ipmt(0.10/12, 1, 3, 8000, 0, FALSE)
/// @export
#[extendr]
fn ipmt(rate: f64, per: f64, nper: f64, pv: f64, fv: f64, pmt_at_beginning: bool) -> f64 {
    if per < 1.0 || per > nper || nper <= 0.0 { return f64::NAN; }
    calc_ipmt(rate, per, nper, pv, fv, pmt_at_beginning)
}

/// Compute the Principal portion of a loan payment for a given period.
///
/// Equivalent to Excel's PPMT function.
///
/// @param rate Interest rate per period (numeric).
/// @param per The period for which to compute principal; must be in [1, nper] (numeric).
/// @param nper Total number of payment periods (numeric).
/// @param pv Present value (numeric).
/// @param fv Future value (numeric, default 0).
/// @param pmt_at_beginning TRUE if payments are due at the beginning of each period.
/// @return Principal payment for the given period. Returns NA if per is outside [1, nper].
/// @examples
/// # Excel: PPMT(0.10/12, 1, 24, 2000, 0, FALSE) = -75.62
/// ppmt(0.10/12, 1, 24, 2000, 0, FALSE)
/// @export
#[extendr]
fn ppmt(rate: f64, per: f64, nper: f64, pv: f64, fv: f64, pmt_at_beginning: bool) -> f64 {
    if per < 1.0 || per > nper || nper <= 0.0 { return f64::NAN; }
    calc_ppmt(rate, per, nper, pv, fv, pmt_at_beginning)
}

/// Compute the Cumulative Interest paid between two periods.
///
/// Equivalent to Excel's CUMIPMT function.
///
/// @param rate Interest rate per period (numeric).
/// @param nper Total number of payment periods (numeric).
/// @param pv Present value (numeric).
/// @param start_period First period in the range; must be >= 1 (numeric).
/// @param end_period Last period in the range; must be >= start_period and <= nper (numeric).
/// @param pmt_at_beginning TRUE if payments are due at the beginning of each period.
/// @return Cumulative interest paid. Returns NA on invalid input.
/// @examples
/// # Excel: CUMIPMT(0.09/12, 360, 125000, 1, 1, FALSE) = -937.50
/// cumipmt(0.09/12, 360, 125000, 1, 1, FALSE)
/// @export
#[extendr]
fn cumipmt(rate: f64, nper: f64, pv: f64, start_period: f64, end_period: f64, pmt_at_beginning: bool) -> f64 {
    if start_period < 1.0 || end_period < start_period || end_period > nper || nper <= 0.0 { return f64::NAN; }
    calc_cumipmt(rate, nper, pv, start_period, end_period, pmt_at_beginning)
}

/// Compute the Cumulative Principal paid between two periods.
///
/// Equivalent to Excel's CUMPRINC function.
///
/// @param rate Interest rate per period (numeric).
/// @param nper Total number of payment periods (numeric).
/// @param pv Present value (numeric).
/// @param start_period First period in the range; must be >= 1 (numeric).
/// @param end_period Last period in the range; must be >= start_period and <= nper (numeric).
/// @param pmt_at_beginning TRUE if payments are due at the beginning of each period.
/// @return Cumulative principal paid. Returns NA on invalid input.
/// @examples
/// # Excel: CUMPRINC(0.09/12, 360, 125000, 1, 1, FALSE) = -68.28
/// cumprinc(0.09/12, 360, 125000, 1, 1, FALSE)
/// @export
#[extendr]
fn cumprinc(rate: f64, nper: f64, pv: f64, start_period: f64, end_period: f64, pmt_at_beginning: bool) -> f64 {
    if start_period < 1.0 || end_period < start_period || end_period > nper || nper <= 0.0 { return f64::NAN; }
    calc_cumprinc(rate, nper, pv, start_period, end_period, pmt_at_beginning)
}

/// Compute the Number of Periods for an investment.
///
/// Equivalent to Excel's NPER function.
///
/// @param rate Interest rate per period (numeric).
/// @param pmt Payment made each period (numeric).
/// @param pv Present value (numeric).
/// @param fv Future value (numeric, default 0).
/// @param pmt_at_beginning TRUE if payments are due at the beginning of each period.
/// @return Number of periods as a double. Returns NA on invalid input.
/// @examples
/// # Excel: NPER(0.12/12, -100, -1000, 10000, TRUE) = 59.67
/// nper(0.12/12, -100, -1000, 10000, TRUE)
/// @export
#[extendr]
fn nper(rate: f64, pmt: f64, pv: f64, fv: f64, pmt_at_beginning: bool) -> f64 {
    calc_nper(rate, pmt, pv, fv, pmt_at_beginning)
}

/// Compute the periodic Interest Rate for an annuity.
///
/// Equivalent to Excel's RATE function. Uses Newton-Raphson iteration.
/// Returns NA if the solver does not converge — try a different guess.
///
/// @param nper Total number of payment periods (numeric, must be > 0).
/// @param pmt Payment made each period (numeric).
/// @param pv Present value (numeric).
/// @param fv Future value (numeric, default 0).
/// @param pmt_at_beginning TRUE if payments are due at the beginning of each period.
/// @param guess Initial guess for the rate (numeric, default 0.1).
/// @return Periodic interest rate as a double. Returns NA if solver fails.
/// @examples
/// # Excel: RATE(48, -200, 8000, 0, FALSE, 0.1) * 12 = 9.24%
/// rate(48, -200, 8000, 0, FALSE, 0.1) * 12
/// @export
#[extendr]
fn rate(nper: f64, pmt: f64, pv: f64, fv: f64, pmt_at_beginning: bool, guess: f64) -> f64 {
    if nper <= 0.0 { return f64::NAN; }
    calc_rate(nper, pmt, pv, fv, pmt_at_beginning, guess)
}

/// Compute the Interest Paid during a specific period of a straight-line loan.
///
/// Equivalent to Excel's ISPMT function.
///
/// @param rate Interest rate per period (numeric).
/// @param per The period number; must be in [1, nper] (numeric).
/// @param nper Total number of payment periods (numeric).
/// @param pv Present value (loan amount) (numeric).
/// @return Interest paid in the given period. Returns NA if per is outside [1, nper].
/// @examples
/// ispmt(0.10/12, 1, 36, 8000000)
/// @export
#[extendr]
fn ispmt(rate: f64, per: f64, nper: f64, pv: f64) -> f64 {
    if nper <= 0.0 || per < 1.0 || per > nper { return f64::NAN; }
    calc_ispmt(rate, per, nper, pv)
}


// ══════════════════════════════════════════════════════════════════════════════
// CASH FLOW FUNCTIONS
// ══════════════════════════════════════════════════════════════════════════════

/// Compute Net Present Value of a series of cash flows.
///
/// Equivalent to Excel's NPV function. Note: Excel's NPV assumes cash flows
/// start at period 1 (not period 0). Add an initial investment separately.
///
/// @param rate Discount rate per period (numeric).
/// @param values Numeric vector of cash flows starting at period 1.
/// @return Net present value. Returns NA on invalid input.
/// @examples
/// # Excel: NPV(0.10, -10000, 3000, 4200, 6800) = 1188.44
/// npv(0.10, c(-10000, 3000, 4200, 6800))
///
/// # With initial investment at time 0:
/// npv(0.08, c(8000, 9200, 10000, 12000, 14500)) + (-40000)
/// @export
#[extendr]
fn npv(rate: f64, values: &[f64]) -> f64 {
    if values.is_empty() { return f64::NAN; }
    npv_calc(rate, values)
}

/// Compute the Internal Rate of Return of a series of cash flows.
///
/// Equivalent to Excel's IRR function. Uses bisection — more robust than
/// Newton-Raphson for non-standard cash flow patterns.
/// Requires at least one positive and one negative cash flow.
///
/// @param values Numeric vector of cash flows. First element is typically a
///   negative investment; subsequent elements are returns.
/// @return IRR as a decimal (e.g. 0.0866 for 8.66%). Returns NA if no solution
///   is found (e.g. all cash flows have the same sign).
/// @examples
/// # Excel: IRR(-70000, 12000, 15000, 18000, 21000, 26000) = 8.66%
/// irr(c(-70000, 12000, 15000, 18000, 21000, 26000))
/// @export
#[extendr]
fn irr(values: &[f64]) -> f64 {
    if values.len() < 2 { return f64::NAN; }
    calc_irr(values)
}

/// Compute the Modified Internal Rate of Return.
///
/// Equivalent to Excel's MIRR function. Uses separate rates for financing
/// and reinvestment. Computed via closed-form formula.
///
/// @param values Numeric vector of cash flows.
/// @param finance_rate Rate paid on negative cash flows (cost of borrowing).
/// @param reinvest_rate Rate earned on positive cash flows (reinvestment return).
/// @return MIRR as a decimal. Returns NA if all cash flows have the same sign.
/// @examples
/// # Excel: MIRR(-120000,39000,30000,21000,37000,46000, 0.10, 0.12) = 12.61%
/// mirr(c(-120000, 39000, 30000, 21000, 37000, 46000), 0.10, 0.12)
/// @export
#[extendr]
fn mirr(values: &[f64], finance_rate: f64, reinvest_rate: f64) -> f64 {
    if values.len() < 2 { return f64::NAN; }
    calc_mirr(values, finance_rate, reinvest_rate)
}

/// Compute Net Present Value with irregular cash flow dates.
///
/// Equivalent to Excel's XNPV function.
///
/// @param rate Discount rate (annual) (numeric).
/// @param values Numeric vector of cash flows.
/// @param dates Character vector of dates in "YYYY-MM-DD" format,
///   same length as values.
/// @return XNPV value. Returns NA if dates cannot be parsed or lengths differ.
/// @examples
/// xnpv(0.09,
///   c(-10000, 2750, 4250, 3250, 2750),
///   c("2008-01-01","2008-03-01","2008-10-30","2009-02-15","2009-04-01"))
/// @export
#[extendr]
fn xnpv(rate: f64, values: &[f64], dates: Vec<String>) -> f64 {
    if values.len() != dates.len() || values.is_empty() { return f64::NAN; }
    let parsed: Vec<NaiveDate> = dates.iter()
        .filter_map(|s| parse_date_str(s))
        .collect();
    if parsed.len() != values.len() { return f64::NAN; }
    let d0 = parsed[0];
    let yf: Vec<f64> = parsed.iter()
        .map(|&d| (d - d0).num_days() as f64 / 365.0)
        .collect();
    xnpv_at(values, &yf, rate)   // ← correct order: values first, year_fracs second, rate third
}

/// Compute the Internal Rate of Return with irregular cash flow dates.
///
/// Equivalent to Excel's XIRR function.
///
/// @param values Numeric vector of cash flows. Must contain at least one
///   positive and one negative value.
/// @param dates Character vector of dates in "YYYY-MM-DD" format,
///   same length as values.
/// @return XIRR as a decimal. Returns NA if no solution is found.
/// @examples
/// xirr(c(-10000, 2750, 4250, 3250, 2750),
///      c("2008-01-01","2008-03-01","2008-10-30","2009-02-15","2009-04-01"))
/// @export
#[extendr]
fn xirr(values: &[f64], dates: Vec<String>) -> f64 {
    if values.len() != dates.len() || values.len() < 2 { return f64::NAN; }
    let parsed: Vec<NaiveDate> = dates.iter()
        .filter_map(|s| parse_date_str(s))
        .collect();
    if parsed.len() != values.len() { return f64::NAN; }
    calc_xirr(values, &parsed)
}


// ══════════════════════════════════════════════════════════════════════════════
// DEPRECIATION FUNCTIONS
// ══════════════════════════════════════════════════════════════════════════════

/// Straight-Line Depreciation.
/// @param cost Initial cost of the asset (numeric).
/// @param salvage Salvage value at end of life (numeric).
/// @param life Number of periods over which the asset is depreciated (numeric).
/// @return Depreciation per period. Returns NA if life = 0.
/// @examples
/// # Excel: SLN(30000, 7500, 10) = 2250
/// sln(30000, 7500, 10)
/// @export
#[extendr]
fn sln(cost: f64, salvage: f64, life: f64) -> f64 { calc_sln(cost, salvage, life) }

/// Sum-of-Years-Digits Depreciation.
/// @param cost Initial cost of the asset (numeric).
/// @param salvage Salvage value at end of life (numeric).
/// @param life Number of periods of useful life (numeric).
/// @param per The period for which depreciation is computed (numeric).
/// @return Depreciation for the given period.
/// @examples
/// # Excel: SYD(30000, 7500, 10, 1) = 4090.91
/// syd(30000, 7500, 10, 1)
/// @export
#[extendr]
fn syd(cost: f64, salvage: f64, life: f64, per: f64) -> f64 { calc_syd(cost, salvage, life, per) }

/// Fixed-Declining Balance Depreciation.
/// @param cost Initial cost of the asset (numeric).
/// @param salvage Salvage value at end of life (numeric).
/// @param life Number of periods of useful life (numeric).
/// @param per The period for which depreciation is computed (numeric).
/// @param month Number of months in the first year (numeric, default 12).
/// @return Depreciation for the given period.
/// @examples
/// # Excel: DB(1000000, 100000, 6, 1, 7) = 186083.33
/// db(1000000, 100000, 6, 1, 7)
/// @export
#[extendr]
fn db(cost: f64, salvage: f64, life: f64, per: f64, month: f64) -> f64 { calc_db(cost, salvage, life, per, month) }

/// Double-Declining Balance Depreciation.
/// @param cost Initial cost of the asset (numeric).
/// @param salvage Salvage value at end of life (numeric).
/// @param life Number of periods of useful life (numeric).
/// @param per The period for which depreciation is computed (numeric).
/// @param factor Declining balance factor (numeric, default 2.0 for double-declining).
/// @return Depreciation for the given period.
/// @examples
/// # Excel: DDB(2400, 300, 10, 1, 2) = 480
/// ddb(2400, 300, 10, 1, 2)
/// @export
#[extendr]
fn ddb(cost: f64, salvage: f64, life: f64, per: f64, factor: f64) -> f64 { calc_ddb(cost, salvage, life, per, factor) }

/// Variable Declining Balance Depreciation between two periods.
/// @param cost Initial cost of the asset (numeric).
/// @param salvage Salvage value at end of life (numeric).
/// @param life Number of periods of useful life (numeric).
/// @param start_period Start of depreciation period (can be fractional) (numeric).
/// @param end_period End of depreciation period (can be fractional) (numeric).
/// @param factor Declining balance factor (numeric, default 2.0).
/// @param no_switch FALSE to switch to straight-line when SL > DDB (Excel default).
/// @return Depreciation between start_period and end_period.
/// @examples
/// # First year depreciation
/// vdb(2400, 300, 10, 0, 1, 2, FALSE)
/// @export
#[extendr]
fn vdb(cost: f64, salvage: f64, life: f64, start_period: f64, end_period: f64, factor: f64, no_switch: bool) -> f64 {
    calc_vdb(cost, salvage, life, start_period, end_period, factor, no_switch)
}

/// French Straight-Line Depreciation (AMORLINC).
/// @param cost Cost of the asset (numeric).
/// @param date_purchased Excel serial date of purchase (numeric).
/// @param first_period Excel serial date of end of first period (numeric).
/// @param salvage Salvage value (numeric).
/// @param period Depreciation period number (numeric).
/// @param rate Annual depreciation rate (numeric).
/// @param basis Day-count basis: 0=30/360, 1=actual/actual, 3=actual/365 (numeric).
/// @return Depreciation for the given period.
/// @examples
/// amorlinc(2400, 39679, 39813, 300, 1, 0.15, 1)
/// @export
#[extendr]
fn amorlinc(cost: f64, date_purchased: f64, first_period: f64, salvage: f64, period: f64, rate: f64, basis: f64) -> f64 {
    calc_amorlinc(cost, date_purchased, first_period, salvage, period, rate, basis)
}

/// French Degressive Depreciation (AMORDEGRC).
/// @param cost Cost of the asset (numeric).
/// @param date_purchased Excel serial date of purchase (numeric).
/// @param first_period Excel serial date of end of first period (numeric).
/// @param salvage Salvage value (numeric).
/// @param period Depreciation period number (numeric).
/// @param rate Annual depreciation rate (numeric).
/// @param basis Day-count basis (numeric).
/// @return Depreciation for the given period. Returns NA for invalid life ranges.
/// @examples
/// amordegrc(2400, 39679, 39813, 300, 1, 0.15, 1)
/// @export
#[extendr]
fn amordegrc(cost: f64, date_purchased: f64, first_period: f64, salvage: f64, period: f64, rate: f64, basis: f64) -> f64 {
    calc_amordegrc(cost, date_purchased, first_period, salvage, period, rate, basis)
}


// ══════════════════════════════════════════════════════════════════════════════
// COUPON DATE FUNCTIONS
// ══════════════════════════════════════════════════════════════════════════════

/// Days from beginning of coupon period to settlement date (COUPDAYBS).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param frequency Coupons per year: 1=annual, 2=semiannual, 4=quarterly (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Number of days. Returns NA if settlement >= maturity or dates are invalid.
/// @examples
/// coupdaybs("2011-01-25", "2011-11-15", 2, 1)
/// @export
#[extendr]
fn coupdaybs(settlement: &str, maturity: &str, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_coupdaybs(s, m, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Total days in the coupon period containing settlement (COUPDAYS).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Number of days in the coupon period. Returns NA on invalid input.
/// @examples
/// coupdays("2011-01-25", "2011-11-15", 2, 1)
/// @export
#[extendr]
fn coupdays(settlement: &str, maturity: &str, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_coupdays(s, m, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Days from settlement to next coupon date (COUPDAYSNC).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Days to next coupon. Returns NA on invalid input.
/// @examples
/// coupdaysnc("2011-01-25", "2011-11-15", 2, 1)
/// @export
#[extendr]
fn coupdaysnc(settlement: &str, maturity: &str, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_coupdaysnc(s, m, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Next coupon date after settlement as an Excel serial number (COUPNCD).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Excel serial date of next coupon. Returns NA on invalid input.
/// @examples
/// coupncd("2011-01-25", "2011-11-15", 2, 1)
/// @export
#[extendr]
fn coupncd(settlement: &str, maturity: &str, frequency: f64, _basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_coupncd(s, m, freq)
        }
        _ => f64::NAN,
    }
}

/// Previous coupon date before settlement as an Excel serial number (COUPPCD).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Excel serial date of previous coupon. Returns NA on invalid input.
/// @examples
/// couppcd("2011-01-25", "2011-11-15", 2, 1)
/// @export
#[extendr]
fn couppcd(settlement: &str, maturity: &str, frequency: f64, _basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_couppcd(s, m, freq)
        }
        _ => f64::NAN,
    }
}

/// Number of coupon payments between settlement and maturity (COUPNUM).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Number of coupon payments remaining. Returns NA on invalid input.
/// @examples
/// coupnum("2007-01-25", "2008-11-15", 2, 1)
/// @export
#[extendr]
fn coupnum(settlement: &str, maturity: &str, frequency: f64, _basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_coupnum(s, m, freq)
        }
        _ => f64::NAN,
    }
}


// ══════════════════════════════════════════════════════════════════════════════
// BOND & SECURITY FUNCTIONS
// ══════════════════════════════════════════════════════════════════════════════

/// Clean price of a bond per $100 face value (PRICE).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param rate Annual coupon rate (numeric, e.g. 0.0575 for 5.75%).
/// @param yld Annual yield (numeric).
/// @param redemption Redemption value per $100 face value (numeric).
/// @param frequency Coupons per year: 1, 2, or 4 (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Clean price per $100 face value. Returns NA on invalid input.
/// @examples
/// # Excel: PRICE("2008-02-15","2017-11-15", 0.0575, 0.065, 100, 2, 0) = 94.63
/// price("2008-02-15", "2017-11-15", 0.0575, 0.065, 100, 2, 0)
/// @export
#[extendr]
fn price(settlement: &str, maturity: &str, rate: f64, yld: f64, redemption: f64, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_price(s, m, rate, yld, redemption, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Price of a discounted security (PRICEDISC).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param discount Annual discount rate (numeric).
/// @param redemption Redemption value (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Price of discounted security. Returns NA on invalid input.
/// @examples
/// pricedisc("2008-02-16", "2008-03-01", 0.0525, 100, 2)
/// @export
#[extendr]
fn pricedisc(settlement: &str, maturity: &str, discount: f64, redemption: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => calc_pricedisc(s, m, discount, redemption, basis as i32),
        _ => f64::NAN,
    }
}

/// Price of a security that pays interest at maturity (PRICEMAT).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param issue Issue date as "YYYY-MM-DD" string.
/// @param rate Annual coupon rate (numeric).
/// @param yld Annual yield (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Price of the security. Returns NA on invalid input.
/// @examples
/// pricemat("2008-02-15", "2008-04-13", "2007-11-11", 0.061, 0.061, 0)
/// @export
#[extendr]
fn pricemat(settlement: &str, maturity: &str, issue: &str, rate: f64, yld: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity), parse_date_str(issue)) {
        (Some(s), Some(m), Some(i)) if s < m => calc_pricemat(s, m, i, rate, yld, basis as i32),
        _ => f64::NAN,
    }
}

/// Yield of a bond (YIELD).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param rate Annual coupon rate (numeric).
/// @param pr Clean price per $100 face value (numeric).
/// @param redemption Redemption value (numeric).
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Annual yield as a decimal. Returns NA on invalid input.
/// @examples
/// yield_("2008-02-15", "2016-11-15", 0.0575, 95.04287, 100, 2, 0)
/// @export
#[extendr]
fn yield_(settlement: &str, maturity: &str, rate: f64, pr: f64, redemption: f64, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_yield(s, m, rate, pr, redemption, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Yield of a discounted security (YIELDDISC).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param pr Price per $100 face value (numeric).
/// @param redemption Redemption value (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Annual yield. Returns NA on invalid input.
/// @examples
/// yielddisc("2008-02-16", "2008-03-01", 99.795, 100, 2)
/// @export
#[extendr]
fn yielddisc(settlement: &str, maturity: &str, pr: f64, redemption: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => calc_yielddisc(s, m, pr, redemption, basis as i32),
        _ => f64::NAN,
    }
}

/// Yield of a security that pays interest at maturity (YIELDMAT).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param issue Issue date as "YYYY-MM-DD" string.
/// @param rate Annual coupon rate (numeric).
/// @param pr Price (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Annual yield. Returns NA on invalid input.
/// @examples
/// yieldmat("2008-03-15", "2008-11-03", "2007-11-08", 0.0625, 100.0123, 0)
/// @export
#[extendr]
fn yieldmat(settlement: &str, maturity: &str, issue: &str, rate: f64, pr: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity), parse_date_str(issue)) {
        (Some(s), Some(m), Some(i)) if s < m => calc_yieldmat(s, m, i, rate, pr, basis as i32),
        _ => f64::NAN,
    }
}

/// Discount rate of a security (DISC).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param pr Price per $100 face value (numeric).
/// @param redemption Redemption value (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Discount rate. Returns NA on invalid input.
/// @examples
/// disc("2008-02-16", "2008-03-01", 97.975, 100, 2)
/// @export
#[extendr]
fn disc(settlement: &str, maturity: &str, pr: f64, redemption: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => calc_disc(s, m, pr, redemption, basis as i32),
        _ => f64::NAN,
    }
}

/// Interest rate for a fully invested security (INTRATE).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param investment Amount invested (numeric).
/// @param redemption Amount received at maturity (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Interest rate. Returns NA on invalid input.
/// @examples
/// intrate("2008-02-15", "2008-05-15", 1000000, 1014420, 2)
/// @export
#[extendr]
fn intrate(settlement: &str, maturity: &str, investment: f64, redemption: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => calc_intrate(s, m, investment, redemption, basis as i32),
        _ => f64::NAN,
    }
}

/// Amount received at maturity for a fully invested security (RECEIVED).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param investment Amount invested (numeric).
/// @param discount Annual discount rate (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Amount received at maturity. Returns NA on invalid input.
/// @examples
/// received("2008-02-15", "2008-05-15", 1000000, 0.0575, 2)
/// @export
#[extendr]
fn received(settlement: &str, maturity: &str, investment: f64, discount: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => calc_received(s, m, investment, discount, basis as i32),
        _ => f64::NAN,
    }
}

/// Macaulay Duration of a bond (DURATION).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param coupon Annual coupon rate (numeric).
/// @param yld Annual yield (numeric).
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Duration in years. Returns NA on invalid input.
/// @examples
/// duration("2008-01-01", "2016-01-01", 0.08, 0.09, 2, 1)
/// @export
#[extendr]
fn duration(settlement: &str, maturity: &str, coupon: f64, yld: f64, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_duration(s, m, coupon, yld, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Modified Duration of a bond (MDURATION).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param coupon Annual coupon rate (numeric).
/// @param yld Annual yield (numeric).
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Modified duration. Returns NA on invalid input.
/// @examples
/// mduration("2008-01-01", "2016-01-01", 0.08, 0.09, 2, 1)
/// @export
#[extendr]
fn mduration(settlement: &str, maturity: &str, coupon: f64, yld: f64, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_mduration(s, m, coupon, yld, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Accrued interest for a periodic-coupon security (ACCRINT).
/// @param issue Issue date as "YYYY-MM-DD" string.
/// @param first_interest First interest date as "YYYY-MM-DD" string.
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param rate Annual coupon rate (numeric).
/// @param par Par value (numeric).
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Accrued interest. Returns NA on invalid input.
/// @examples
/// accrint("2008-03-01", "2008-08-31", "2008-05-01", 0.10, 1000, 2, 0)
/// @export
#[extendr]
fn accrint(issue: &str, first_interest: &str, settlement: &str, rate: f64, par: f64, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(issue), parse_date_str(first_interest), parse_date_str(settlement)) {
        (Some(i), Some(f), Some(s)) => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_accrint(i, f, s, rate, par, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Accrued interest for a security that pays at maturity (ACCRINTM).
/// @param issue Issue date as "YYYY-MM-DD" string.
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param rate Annual coupon rate (numeric).
/// @param par Par value (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Accrued interest. Returns NA on invalid input.
/// @examples
/// accrintm("2008-04-01", "2008-06-15", 0.10, 1000, 3)
/// @export
#[extendr]
fn accrintm(issue: &str, settlement: &str, rate: f64, par: f64, basis: f64) -> f64 {
    match (parse_date_str(issue), parse_date_str(settlement)) {
        (Some(i), Some(s)) => calc_accrintm(i, s, rate, par, basis as i32),
        _ => f64::NAN,
    }
}

/// Price of a bond with an odd first period (ODDFPRICE).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param issue Issue date as "YYYY-MM-DD" string.
/// @param first_coupon First coupon date as "YYYY-MM-DD" string.
/// @param rate Annual coupon rate (numeric).
/// @param yld Annual yield (numeric).
/// @param redemption Redemption value (numeric).
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Price of the bond. Returns NA on invalid input.
/// @examples
/// oddfprice("2008-11-11","2021-03-01","2008-10-15","2009-03-01",0.0785,0.0625,100,2,1)
/// @export
#[extendr]
fn oddfprice(settlement: &str, maturity: &str, issue: &str, first_coupon: &str, rate: f64, yld: f64, redemption: f64, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity), parse_date_str(issue), parse_date_str(first_coupon)) {
        (Some(s), Some(m), Some(i), Some(f)) => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_oddfprice(s, m, i, f, rate, yld, redemption, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Yield of a bond with an odd first period (ODDFYIELD).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param issue Issue date as "YYYY-MM-DD" string.
/// @param first_coupon First coupon date as "YYYY-MM-DD" string.
/// @param rate Annual coupon rate (numeric).
/// @param pr Price (numeric).
/// @param redemption Redemption value (numeric).
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Yield as a decimal. Returns NA on invalid input.
/// @examples
/// oddfyield("2008-11-11","2021-03-01","2008-10-15","2009-03-01",0.0575,84.50,100,2,1)
/// @export
#[extendr]
fn oddfyield(settlement: &str, maturity: &str, issue: &str, first_coupon: &str, rate: f64, pr: f64, redemption: f64, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity), parse_date_str(issue), parse_date_str(first_coupon)) {
        (Some(s), Some(m), Some(i), Some(f)) => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_oddfyield(s, m, i, f, rate, pr, redemption, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Price of a bond with an odd last period (ODDLPRICE).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param last_interest Last interest date as "YYYY-MM-DD" string.
/// @param rate Annual coupon rate (numeric).
/// @param yld Annual yield (numeric).
/// @param redemption Redemption value (numeric).
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Price of the bond. Returns NA on invalid input.
/// @examples
/// oddlprice("2008-02-07","2008-06-15","2007-10-15",0.0375,0.0405,100,2,0)
/// @export
#[extendr]
fn oddlprice(settlement: &str, maturity: &str, last_interest: &str, rate: f64, yld: f64, redemption: f64, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity), parse_date_str(last_interest)) {
        (Some(s), Some(m), Some(l)) => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_oddlprice(s, m, l, rate, yld, redemption, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}

/// Yield of a bond with an odd last period (ODDLYIELD).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param last_interest Last interest date as "YYYY-MM-DD" string.
/// @param rate Annual coupon rate (numeric).
/// @param pr Price (numeric).
/// @param redemption Redemption value (numeric).
/// @param frequency Coupons per year (numeric).
/// @param basis Day-count basis 0-4 (numeric).
/// @return Yield as a decimal. Returns NA on invalid input.
/// @examples
/// oddlyield("2008-04-20","2008-06-15","2007-12-24",0.0375,99.875,100,2,0)
/// @export
#[extendr]
fn oddlyield(settlement: &str, maturity: &str, last_interest: &str, rate: f64, pr: f64, redemption: f64, frequency: f64, basis: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity), parse_date_str(last_interest)) {
        (Some(s), Some(m), Some(l)) => {
            let freq = helpers::freq_per_year(frequency as i32);
            calc_oddlyield(s, m, l, rate, pr, redemption, freq, basis as i32)
        }
        _ => f64::NAN,
    }
}


// ══════════════════════════════════════════════════════════════════════════════
// MISCELLANEOUS FUNCTIONS
// ══════════════════════════════════════════════════════════════════════════════

/// Effective annual interest rate (EFFECT).
/// @param nominal_rate Nominal annual interest rate (numeric).
/// @param npery Number of compounding periods per year (numeric).
/// @return Effective annual rate. Returns NA if npery = 0.
/// @examples
/// effect(0.0525, 4)  # quarterly compounding
/// @export
#[extendr]
fn effect(nominal_rate: f64, npery: f64) -> f64 {
    if npery == 0.0 { return f64::NAN; }
    (1.0 + nominal_rate / npery).powf(npery) - 1.0
}

/// Nominal annual interest rate (NOMINAL).
/// @param effect_rate Effective annual interest rate (numeric).
/// @param npery Number of compounding periods per year (numeric).
/// @return Nominal annual rate. Returns NA if npery = 0.
/// @examples
/// nominal(0.053543, 4)
/// @export
#[extendr]
fn nominal(effect_rate: f64, npery: f64) -> f64 {
    if npery == 0.0 { return f64::NAN; }
    ((1.0 + effect_rate).powf(1.0 / npery) - 1.0) * npery
}

/// Convert a dollar price in fractional notation to decimal (DOLLARDE).
/// @param fractional_dollar Price in fractional notation (numeric, e.g. 1.02 = 1 + 2/16).
/// @param fraction Denominator of the fraction (numeric, e.g. 16 for sixteenths).
/// @return Decimal price. Returns NA if fraction = 0.
/// @examples
/// dollarde(1.02, 16)  # = 1.125
/// @export
#[extendr]
fn dollarde(fractional_dollar: f64, fraction: f64) -> f64 {
    let f = fraction.floor();
    if f == 0.0 { return f64::NAN; }
    let integer_part = fractional_dollar.floor();
    let decimal_part = fractional_dollar - integer_part;
    integer_part + decimal_part / f * 10.0f64.powf(f.log10().ceil())
}

/// Convert a decimal dollar price to fractional notation (DOLLARFR).
/// @param decimal_dollar Price as a decimal (numeric).
/// @param fraction Denominator of the fraction (numeric).
/// @return Fractional price. Returns NA if fraction = 0.
/// @examples
/// dollarfr(1.125, 16)  # = 1.02
/// @export
#[extendr]
fn dollarfr(decimal_dollar: f64, fraction: f64) -> f64 {
    let f = fraction.floor();
    if f == 0.0 { return f64::NAN; }
    let integer_part = decimal_dollar.floor();
    let frac_part = decimal_dollar - integer_part;
    integer_part + frac_part * f / 10.0f64.powf(f.log10().ceil())
}

/// Future value of an investment with a variable rate schedule (FVSCHEDULE).
/// @param principal Initial principal amount (numeric).
/// @param schedule Numeric vector of interest rates to apply sequentially.
/// @return Future value after applying all rates. Returns NA if schedule is empty.
/// @examples
/// fvschedule(1, c(0.09, 0.11, 0.10))  # = 1.3309
/// @export
#[extendr]
fn fvschedule(principal: f64, schedule: &[f64]) -> f64 {
    if schedule.is_empty() { return f64::NAN; }
    schedule.iter().fold(principal, |acc, &r| acc * (1.0 + r))
}

/// Equivalent interest rate for investment growth (RRI).
/// @param nper Number of periods (numeric, must be > 0).
/// @param pv Present value (numeric, must not be 0).
/// @param fv Future value (numeric).
/// @return Equivalent periodic interest rate. Returns NA on invalid input.
/// @examples
/// rri(96, 10000, 11000)
/// @export
#[extendr]
fn rri(nper: f64, pv: f64, fv: f64) -> f64 {
    if nper == 0.0 || pv == 0.0 { return f64::NAN; }
    (fv / pv).powf(1.0 / nper) - 1.0
}

/// Number of periods to reach a target value (PDURATION).
/// @param rate Periodic interest rate (numeric).
/// @param pv Present value (numeric, must not be 0).
/// @param fv Future value (numeric).
/// @return Number of periods. Returns NA on invalid input.
/// @examples
/// pduration(0.025, 2000, 2200)  # = 3.86
/// @export
#[extendr]
fn pduration(rate: f64, pv: f64, fv: f64) -> f64 {
    if pv == 0.0 || rate <= -1.0 { return f64::NAN; }
    let ln_denom = (1.0 + rate).ln();
    if ln_denom.abs() < 1e-12 { return f64::NAN; }
    (fv / pv).ln() / ln_denom
}

/// Treasury bill price per $100 face value (TBILLPRICE).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param discount Annual discount rate (numeric).
/// @return T-bill price. Returns NA on invalid input.
/// @examples
/// tbillprice("2008-03-31", "2008-06-01", 0.09)  # = 98.45
/// @export
#[extendr]
fn tbillprice(settlement: &str, maturity: &str, discount: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let days = (m - s).num_days() as f64;
            100.0 * (1.0 - discount * days / 360.0)
        }
        _ => f64::NAN,
    }
}

/// Treasury bill yield (TBILLYIELD).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param pr T-bill price per $100 face value (numeric).
/// @return Annual yield. Returns NA on invalid input.
/// @examples
/// tbillyield("2008-03-31", "2008-06-01", 98.45)  # = 0.0914
/// @export
#[extendr]
fn tbillyield(settlement: &str, maturity: &str, pr: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let days = (m - s).num_days() as f64;
            if pr == 0.0 || days == 0.0 { return f64::NAN; }
            (100.0 - pr) / pr * 360.0 / days
        }
        _ => f64::NAN,
    }
}

/// Treasury bill bond-equivalent yield (TBILLEQ).
/// @param settlement Settlement date as "YYYY-MM-DD" string.
/// @param maturity Maturity date as "YYYY-MM-DD" string.
/// @param discount Annual discount rate (numeric).
/// @return Bond-equivalent yield. Returns NA on invalid input.
/// @examples
/// tbilleq("2008-03-31", "2008-06-01", 0.0914)  # = 0.0942
/// @export
#[extendr]
fn tbilleq(settlement: &str, maturity: &str, discount: f64) -> f64 {
    match (parse_date_str(settlement), parse_date_str(maturity)) {
        (Some(s), Some(m)) if s < m => {
            let days = (m - s).num_days() as f64;
            if days <= 0.0 { return f64::NAN; }
            if days <= 182.0 {
                let denom = 360.0 - discount * days;
                if denom.abs() < 1e-12 { return f64::NAN; }
                365.0 * discount / denom
            } else {
                let a = discount * days / 360.0;
                let denom = 1.0 - a;
                if denom.abs() < 1e-12 { return f64::NAN; }
                let inner = (2.0 * a / denom + 1.0).sqrt();
                (2.0 * (inner - 1.0)) * 365.0 / days
            }
        }
        _ => f64::NAN,
    }
}


// ── Register all 55 functions with R ─────────────────────────────────────────
extendr_module! {
    mod financial_functions;
    // Annuity
    fn fv;
    fn pv;
    fn pmt;
    fn ipmt;
    fn ppmt;
    fn cumipmt;
    fn cumprinc;
    fn nper;
    fn rate;
    fn ispmt;
    // Cash flows
    fn npv;
    fn irr;
    fn mirr;
    fn xnpv;
    fn xirr;
    // Depreciation
    fn sln;
    fn syd;
    fn db;
    fn ddb;
    fn vdb;
    fn amorlinc;
    fn amordegrc;
    // Coupon dates
    fn coupdaybs;
    fn coupdays;
    fn coupdaysnc;
    fn coupncd;
    fn couppcd;
    fn coupnum;
    // Bonds
    fn price;
    fn pricedisc;
    fn pricemat;
    fn yield_;
    fn yielddisc;
    fn yieldmat;
    fn disc;
    fn intrate;
    fn received;
    fn duration;
    fn mduration;
    fn accrint;
    fn accrintm;
    fn oddfprice;
    fn oddfyield;
    fn oddlprice;
    fn oddlyield;
    // Misc
    fn effect;
    fn nominal;
    fn dollarde;
    fn dollarfr;
    fn fvschedule;
    fn rri;
    fn pduration;
    fn tbillprice;
    fn tbillyield;
    fn tbilleq;
}
