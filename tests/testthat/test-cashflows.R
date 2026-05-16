test_that("npv matches Excel examples", {
  expect_equal(round(npv(0.10, c(-10000, 3000, 4200, 6800)), 2), 1188.44)
  expect_equal(round(npv(0.08, c(8000, 9200, 10000, 12000, 14500)) + (-40000), 2), 1922.06)
  expect_true(is.nan(npv(0.10, numeric(0))))  # empty
})

test_that("irr matches Excel examples", {
  expect_equal(round(irr(c(-70000, 12000, 15000, 18000, 21000, 26000)), 4), 0.0866)
  expect_equal(round(irr(c(-70000, 12000, 15000, 18000, 21000)), 4), -0.0212)
  expect_equal(round(irr(c(-70000, 12000, 15000)), 4), -0.4435)
  expect_true(is.nan(irr(c(100, 200, 300))))    # all positive
  expect_true(is.nan(irr(c(-100, -200, -300)))) # all negative
  expect_true(is.nan(irr(c(-1000))))            # single value
})

test_that("mirr matches Excel examples", {
  expect_equal(round(mirr(c(-120000,39000,30000,21000,37000,46000), 0.10, 0.12), 4), 0.1261)
  expect_equal(round(mirr(c(-120000,39000,30000,21000),             0.10, 0.12), 4), -0.0480)
  expect_equal(round(mirr(c(-120000,39000,30000,21000,37000,46000), 0.10, 0.14), 4), 0.1348)
  expect_true(is.nan(mirr(c(100, 200, 300), 0.10, 0.12)))  # all positive
})

test_that("xnpv matches Excel examples", {
  result <- xnpv(0.09,
    c(-10000, 2750, 4250, 3250, 2750),
    c("2008-01-01","2008-03-01","2008-10-30","2009-02-15","2009-04-01"))
  expect_equal(round(result, 2), 2086.65)
  expect_true(is.nan(xnpv(0.09, c(-1000, 500), c("2020-01-01"))))  # length mismatch
})

test_that("xirr matches Excel examples", {
  result <- xirr(
    c(-10000, 2750, 4250, 3250, 2750),
    c("2008-01-01","2008-03-01","2008-10-30","2009-02-15","2009-04-01"))
  expect_equal(round(result, 4), 0.3734)
  expect_true(is.nan(xirr(c(100, 200, 300), c("2020-01-01","2020-06-01","2021-01-01"))))
})
