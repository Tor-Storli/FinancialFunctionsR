# FinancialFunctions

**55 Excel-compatible financial functions powered by Rust**

<!-- badges: start -->
[![R-CMD-check](https://github.com/Tor-Storli/FinancialFunctionsR/actions/workflows/R-CMD-check.yaml/badge.svg)](https://github.com/Tor-Storli/FinancialFunctionsR/actions/workflows/R-CMD-check.yaml)
<!-- badges: end -->

**FinancialFunctions** provides 55 Excel-compatible financial functions 
implemented in Rust via [extendr](https://extendr.github.io), plus 
6 batch variants for processing large datasets efficiently. All scalar 
functions match Excel output exactly and return `NA` on invalid input.

## Installation

### Prerequisites

This package compiles Rust code from source. Before installing you need:

**Windows:**
1. [Rust](https://rustup.rs) — download and run `rustup-init.exe`   
2. [Rtools45](https://cran.r-project.org/bin/windows/Rtools/rtools45/rtools.html)

**macOS:**
1. [Rust](https://rustup.rs) — run in Terminal:   
   `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Install from GitHub

```r
remotes::install_github("Tor-Storli/FinancialFunctionsR",
                         build_vignettes = TRUE)
```

After installation, browse the vignette:

```r
browseVignettes("FinancialFunctions")
```

### Verify installation

```r
library(FinancialFunctions)
pmt(0.0325/12, 180, 350000, 0, FALSE)
#> [1] -2459.341
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
## Batch Functions — High Performance Processing

For processing large numbers of calculations, use the batch variants which
cross the R-Rust boundary only once instead of once per calculation:

| Batch Function | Description |
|---|---|
| `irr_batch(values_list)` | IRR for a list of cash flow vectors |
| `xirr_batch(values_list, dates_list)` | XIRR for a list of flows and dates |
| `npv_batch(rate, values_list)` | NPV for a list of cash flow vectors |
| `mirr_batch(values_list, finance_rate, reinvest_rate)` | MIRR for a list of flows |
| `pmt_vec(rate, nper, pv, fv, pmt_at_beginning)` | PMT for vectorised inputs |
| `price_curve(settlement, maturity, rate, ylds, redemption, frequency, basis)` | Bond prices across a yield curve |

```r
# Generate 10,000 random investment scenarios
set.seed(42)
flows_10k <- lapply(1:10000, function(i) {
  investment <- -sample(50000:200000, 1)
  returns    <- runif(6, 5000, 50000)
  c(investment, returns)
})

# Calculate all 10,000 IRRs in one Rust call
system.time(
  results <- irr_batch(flows_10k)
)

# Summary of results
summary(results)

# Price/yield curve — 100,000 yield levels in one call
yields <- seq(0.01, 0.15, length.out = 100000)
prices <- price_curve("2024-01-01", "2034-01-01", 0.05, yields, 100, 2, 0)
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

- **Getting Started:** <https://tor-storli.github.io/FinancialFunctionsR/articles/FinancialFunctions.html>
- **Function Reference:** <https://tor-storli.github.io/FinancialFunctionsR/reference/>
- **Package Website:** <https://tor-storli.github.io/FinancialFunctionsR/>

## License

MIT © Tor Storli

Built with [extendr](https://extendr.github.io) — the R-Rust bridge.
