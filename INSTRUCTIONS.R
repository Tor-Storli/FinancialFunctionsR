# Step-by-Step: Building the FinancialFunctions R Package
# =========================================================
# This guide walks through every step from cloning your existing
# DuckDB extension to a fully working R package.
# =========================================================


# ══════════════════════════════════════════════════════════════
# PREREQUISITES — Install once on your machine
# ══════════════════════════════════════════════════════════════

# 1. Rust (you already have this)
#    Verify:
#      rustc --version

# 2. R packages needed
install.packages(c("devtools", "usethis", "rextendr", "testthat", "remotes"))

# 3. On Windows: Rtools must be installed.
#    Download from: https://cran.r-project.org/bin/windows/Rtools/
#    Make sure it matches your R version (e.g. Rtools44 for R 4.4.x)


# ══════════════════════════════════════════════════════════════
# STEP 1 — Create the R package folder
# ══════════════════════════════════════════════════════════════

library(usethis)
library(rextendr)

# Choose where to create the package (change path to suit you)
usethis::create_package("C:/Users/storl/Desktop/FinancialFunctions")

# This opens the new package in a new RStudio window.
# All remaining steps run INSIDE that new project.


# ══════════════════════════════════════════════════════════════
# STEP 2 — Set up the Rust scaffolding inside the R package
# ══════════════════════════════════════════════════════════════

# Run this inside the new FinancialFunctions RStudio project:
rextendr::use_extendr()

# This creates:
#   src/rust/Cargo.toml       <- you will replace this
#   src/rust/src/lib.rs       <- you will replace this
#   R/extendr-wrappers.R      <- auto-generated, never edit manually
#   src/Makevars              <- auto-generated
#   src/Makevars.win          <- auto-generated


# ══════════════════════════════════════════════════════════════
# STEP 3 — Copy the files from this guide into the package
# ══════════════════════════════════════════════════════════════

# Replace these auto-generated files with the ones provided:
#
#   DESCRIPTION               <- replace with provided DESCRIPTION
#   NAMESPACE                 <- replace with provided NAMESPACE
#   LICENSE                   <- copy provided LICENSE file
#   README.md                 <- copy provided README.md
#   src/rust/Cargo.toml       <- replace with provided Cargo.toml
#   src/rust/src/lib.rs       <- replace with provided lib.rs (all 55 wrappers)
#   R/financial_functions-package.R  <- new file, copy as-is
#   tests/testthat.R          <- replace with provided version
#   tests/testthat/test-annuity.R    <- new file
#   tests/testthat/test-cashflows.R  <- new file
#   tests/testthat/test-other.R      <- new file


# ══════════════════════════════════════════════════════════════
# STEP 4 — Copy your existing Rust calculation files
# ══════════════════════════════════════════════════════════════

# Clone your DuckDB extension repo:
#   git clone https://github.com/Tor-Storli/Financial_Functions.git
#
# From that repo, copy these files into src/rust/src/ of your R package:
#
#   helpers.rs
#   annuity.rs
#   cash_flows.rs
#   depreciation.rs
#   coupons.rs
#   bonds.rs
#   misc.rs
#
# IMPORTANT — Edit each copied file:
#
# (a) Remove ALL lines that start with:
#       use duckdb::
#       use libduckdb_sys::
#
# (b) Remove ALL struct definitions and impl VScalar blocks
#     (everything after the last `pub fn calc_*` function in each file)
#     Keep ONLY the `pub fn calc_*` functions.
#
# (c) In helpers.rs specifically:
#     - Remove the `write_f64` function (not needed in R — NaN is NA)
#     - Remove the `read_varchar` function (not needed in R)
#     - Keep: add_months, days_in_month, is_leap, year_frac, freq_per_year,
#             parse_csv_f64, parse_csv_dates, parse_date
#
# Example: annuity.rs before and after
#
# BEFORE (DuckDB version):
#   use duckdb::core::{DataChunkHandle, ...};   <- DELETE this line
#   use duckdb::vscalar::{...};                  <- DELETE this line
#   use crate::helpers::write_f64;               <- DELETE this line
#   pub fn calc_pmt(...) -> f64 { ... }         <- KEEP this
#   pub struct PmtFunction;                      <- DELETE from here
#   impl VScalar for PmtFunction { ... }         <- DELETE to end of file
#
# AFTER (R package version):
#   pub fn calc_pmt(...) -> f64 { ... }         <- only this remains


# ══════════════════════════════════════════════════════════════
# STEP 5 — Also update cash_flows.rs
# ══════════════════════════════════════════════════════════════

# The R package lib.rs calls these internal functions from cash_flows.rs:
#   calc_npv(rate: f64, values: &[f64]) -> f64
#   calc_irr(values: &[f64]) -> f64
#   calc_mirr(values: &[f64], finance_rate: f64, reinvest_rate: f64) -> f64
#   calc_xnpv(rate: f64, values: &[f64], dates: &[NaiveDate]) -> f64
#   calc_xirr(values: &[f64], dates: &[NaiveDate]) -> f64
#
# Make sure these function signatures exist in cash_flows.rs with pub visibility.
# They already exist in your DuckDB extension version — just keep them.


# ══════════════════════════════════════════════════════════════
# STEP 6 — Also update misc.rs
# ══════════════════════════════════════════════════════════════

# The lib.rs uses these from misc.rs:
#   calc_oddfprice(...)
#   calc_oddfyield(...)
#   calc_oddlprice(...)
#   calc_oddlyield(...)
#
# And these from bonds.rs:
#   calc_price, calc_pricedisc, calc_pricemat, calc_yield,
#   calc_yielddisc, calc_yieldmat, calc_disc, calc_intrate,
#   calc_received, calc_duration, calc_mduration,
#   calc_accrint, calc_accrintm
#
# All already exist in your repo — keep the pub fn, remove VScalar structs.


# ══════════════════════════════════════════════════════════════
# STEP 7 — Build and generate R wrappers
# ══════════════════════════════════════════════════════════════

# Inside RStudio with the FinancialFunctions project open:
rextendr::document()

# This does three things:
#   1. Compiles all your Rust code (takes 1-2 minutes first time)
#   2. Generates R/extendr-wrappers.R automatically
#   3. Generates man/ documentation files from your /// comments

# If you see Rust compile errors, they will appear in the console.
# Most common errors are missing `pub` on calc_* functions or
# remaining `use duckdb::` imports that weren't deleted.


# ══════════════════════════════════════════════════════════════
# STEP 8 — Load and test interactively
# ══════════════════════════════════════════════════════════════

devtools::load_all()

# Now test each function group:

# Annuity
pmt(0.0325/12, 180, 350000, 0, FALSE)       # -2459.34
fv(0.06/12, 10, -200, -500, TRUE)           # 2581.40
rate(48, -200, 8000, 0, FALSE, 0.1) * 12   # 0.0924

# Cash flows
irr(c(-70000, 12000, 15000, 18000, 21000, 26000))              # 0.0866
mirr(c(-120000, 39000, 30000, 21000, 37000, 46000), 0.10, 0.12) # 0.1261
npv(0.10, c(-10000, 3000, 4200, 6800))                          # 1188.44

xirr(c(-10000, 2750, 4250, 3250, 2750),
     c("2008-01-01","2008-03-01","2008-10-30","2009-02-15","2009-04-01"))
# 0.3734

# Bonds
price("2008-02-15", "2017-11-15", 0.0575, 0.065, 100, 2, 0)   # 94.63
yield_("2008-02-15", "2016-11-15", 0.0575, 95.04287, 100, 2, 0) # 0.065

# Depreciation
sln(30000, 7500, 10)                     # 2250
ddb(2400, 300, 10, 1, 2)                 # 480
amordegrc(2400, 39679, 39813, 300, 1, 0.15, 1)  # 776

# Coupons
coupdaybs("2011-01-25", "2011-11-15", 2, 1)  # 71
coupdays("2011-01-25", "2011-11-15", 2, 1)   # 181

# Misc
effect(0.0525, 4)      # 0.053543
tbilleq("2008-03-31", "2008-06-01", 0.0914)  # 0.0942


# ══════════════════════════════════════════════════════════════
# STEP 9 — Run the test suite
# ══════════════════════════════════════════════════════════════

devtools::test()

# Expected output:
# ✓ | OK F W S | Context
# ✓ | 22       | annuity functions [1.2s]
# ✓ | 12       | cash flow functions [0.8s]
# ✓ | 25       | other functions [0.5s]


# ══════════════════════════════════════════════════════════════
# STEP 10 — Run CRAN checks
# ══════════════════════════════════════════════════════════════

devtools::check()

# Fix any warnings or errors before publishing.
# Common ones to fix:
#   - Missing Rd documentation → add /// comments to Rust functions
#   - NOTE about non-standard files → normal for Rust packages
#   - WARNING about SystemRequirements → already handled in DESCRIPTION


# ══════════════════════════════════════════════════════════════
# STEP 11 — Publish to GitHub
# ══════════════════════════════════════════════════════════════

# From RStudio terminal:
#   git init
#   git add .
#   git commit -m "Initial R package release"
#   git remote add origin https://github.com/Tor-Storli/FinancialFunctions.git
#   git push -u origin main

# Users can then install with:
#   remotes::install_github("Tor-Storli/FinancialFunctions")


# ══════════════════════════════════════════════════════════════
# STEP 12 (Optional) — Submit to CRAN
# ══════════════════════════════════════════════════════════════

# Before submitting, run the full CRAN check suite:
devtools::check(cran = TRUE)

# Then submit:
devtools::release()

# CRAN requirements for Rust packages:
#   1. Rust must be listed in SystemRequirements in DESCRIPTION (done)
#   2. The package must compile from source on all platforms
#   3. All examples must run in < 5 seconds
#   4. No internet access during checks
#
# Note: CRAN may ask you to vendor Rust dependencies
# (bundle them in the package rather than downloading at install time).
# If they ask, run:  cargo vendor  inside src/rust/ and follow CRAN guidance.


# ══════════════════════════════════════════════════════════════
# TROUBLESHOOTING
# ══════════════════════════════════════════════════════════════

# Problem: rextendr::document() fails with "cargo not found"
# Fix: Make sure Rust is on your PATH
#   In R: Sys.setenv(PATH = paste(Sys.getenv("PATH"), "C:/Users/storl/.cargo/bin", sep=";"))

# Problem: "use of undeclared crate or module `duckdb`"
# Fix: You missed removing a `use duckdb::` line from one of the .rs files

# Problem: "cannot find function `write_f64`"
# Fix: Remove `use crate::helpers::write_f64;` from the module files
#      The R version doesn't use write_f64

# Problem: "error[E0432]: unresolved import `crate::helpers::read_varchar`"
# Fix: Remove that import — read_varchar is DuckDB-specific, not needed in R

# Problem: Functions return NaN in R
# Fix: This is correct — NaN displays as NA in R, meaning invalid input was caught

# Problem: `yield` name conflict
# Fix: The function is named `yield_` in this package to avoid R's keyword conflict
