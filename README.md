# FinancialFunctions

**55 Excel-compatible financial functions powered by Rust**

<!-- badges: start -->
[![R-CMD-check](https://github.com/Tor-Storli/FinancialFunctionsR/actions/workflows/R-CMD-check.yaml/badge.svg)](https://github.com/Tor-Storli/FinancialFunctionsR/actions/workflows/R-CMD-check.yaml)
<!-- badges: end -->

**FinancialFunctions** is an R package providing 55 Excel-compatible financial
functions implemented in Rust via [extendr](https://extendr.github.io). All
functions match Excel output exactly and handle invalid inputs gracefully by
returning `NA` rather than throwing errors.

## Installation

```r
# Install from GitHub (requires remotes)
remotes::install_github("Tor-Storli/FinancialFunctionsR")
```

Rust must be installed on your system. Install it from <https://rustup.rs>.

## Function Groups

| Group | Functions |
|---|---|
| **Annuity** | `fv`, `pv`, `pmt`, `ipmt`, `ppmt`, `cumipmt`, `cumprinc`, `nper`, `rate`, `ispmt` |
| **Cash Flows** | `npv`, `irr`, `mirr`, `xnpv`, `xirr` |
| **Depreciation** | `sln`, `syd`, `db`, `ddb`, `vdb`, `amorlinc`, `amordegrc` |
| **Coupon Dates** | `coupdaybs`, `coupdays`, `coupdaysnc`, `coupncd`, `couppcd`, `coupnum` |
| **Bonds & Securities** | `price`, `pricedisc`, `pricemat`, `yield_`, `yielddisc`, `yieldmat`, `disc`, `intrate`, `received`, `duration`, `mduration`, `accrint`, `accrintm`, `oddfprice`, `oddfyield`, `oddlprice`, `oddlyield` |
| **Miscellaneous** | `effect`, `nominal`, `dollarde`, `dollarfr`, `fvschedule`, `rri`, `pduration`, `tbillprice`, `tbillyield`, `tbilleq` |

> **Note:** The `yield` function is named `yield_` in R to avoid a conflict
> with R's built-in `yield` keyword.

## Usage

```r
library(FinancialFunctions)

# Future value of saving $200/month for 10 years at 5% annual interest
fv(0.05/12, 120, -200, 0, FALSE)
#> [1] 31056.46

# Monthly payment on a $350,000 mortgage at 3.25% over 15 years
pmt(0.0325/12, 180, 350000, 0, FALSE)
#> [1] -2459.34

# IRR of a 5-year investment
irr(c(-70000, 12000, 15000, 18000, 21000, 26000))
#> [1] 0.0866

# MIRR with separate finance and reinvestment rates
mirr(c(-120000, 39000, 30000, 21000, 37000, 46000), 0.10, 0.12)
#> [1] 0.1261

# XIRR with irregular cash flow dates
xirr(
  c(-10000, 2750, 4250, 3250, 2750),
  c("2008-01-01", "2008-03-01", "2008-10-30", "2009-02-15", "2009-04-01")
)
#> [1] 0.3734

# Bond price (clean price per $100 face value)
price("2008-02-15", "2017-11-15", 0.0575, 0.065, 100, 2, 0)
#> [1] 94.63
```

## Key Differences from Excel

| Excel | R |
|---|---|
| `YIELD(...)` | `yield_(...)` (trailing underscore) |
| `IRR(A1:A6)` | `irr(c(-70000, 12000, ...))` |
| `NPV(rate, A1:A5)` | `npv(rate, c(cf1, cf2, ...))` |
| `XIRR(A1:A5, B1:B5)` | `xirr(values, c("2024-01-01", ...))` |
| `TRUE`/`FALSE` | `TRUE`/`FALSE` (same) |
| `#NUM!` error | `NA` |

## 📺 Tutorial

A step-by-step Quarto tutorial covering how this package was built
(from a DuckDB Rust extension to a native R package) is available in
the [`tutorials/`](tutorials/) folder.

Render it in RStudio:

```r
quarto::quarto_render("tutorials/FinancialFunctions_Tutorial.qmd")
```

## Documentation

Full function reference and getting started guide:
<https://tor-storli.github.io/FinancialFunctionsR/>

## License

MIT © Tor Storli

Built with [extendr](https://extendr.github.io) — the R-Rust bridge.
