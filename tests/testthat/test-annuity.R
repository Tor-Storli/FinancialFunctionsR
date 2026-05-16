test_that("fv matches Excel examples", {
  expect_equal(round(fv(0.06/12, 10, -200, -500, TRUE),  2), 2581.40)
  expect_equal(round(fv(0.12/12, 12, -1000,  0, FALSE), 2), 12682.50)
  expect_equal(round(fv(0.0,     10, -100, -500, FALSE), 2), 1500.00)
})

test_that("pv matches Excel examples", {
  expect_equal(round(pv(0.08/12, 240, 500, 0, FALSE), 2), -59777.15)
})

test_that("pmt matches Excel examples", {
  expect_equal(round(pmt(0.08/12, 10, 10000, 0, FALSE), 2), -1037.03)
  expect_equal(round(pmt(0.0,     12, 1200,  0, FALSE), 2), -100.00)
})

test_that("ipmt matches Excel and guards bad input", {
  expect_equal(round(ipmt(0.10/12, 1, 3, 8000, 0, FALSE), 2), -66.67)
  expect_true(is.nan(ipmt(0.10/12, 0,  12, 10000, 0, FALSE)))  # per < 1
  expect_true(is.nan(ipmt(0.10/12, 99, 12, 10000, 0, FALSE)))  # per > nper
  expect_true(is.nan(ipmt(0.10/12, 1,   0, 10000, 0, FALSE)))  # nper = 0
})

test_that("ppmt matches Excel and guards bad input", {
  expect_equal(round(ppmt(0.10/12, 1, 24, 2000, 0, FALSE), 2), -75.62)
  expect_true(is.nan(ppmt(0.10/12, 0, 12, 10000, 0, FALSE)))
})

test_that("cumipmt matches Excel and guards bad input", {
  expect_equal(round(cumipmt(0.09/12, 360, 125000, 1, 1, FALSE), 2), -937.50)
  expect_true(is.nan(cumipmt(0.09/12, 360, 125000, 5,   1, FALSE)))  # start > end
  expect_true(is.nan(cumipmt(0.09/12, 360, 125000, 0,  12, FALSE)))  # start < 1
  expect_true(is.nan(cumipmt(0.09/12,  12, 125000, 1, 999, FALSE)))  # end > nper
})

test_that("cumprinc matches Excel and guards bad input", {
  expect_equal(round(cumprinc(0.09/12, 360, 125000, 1, 1, FALSE), 2), -68.28)
  expect_true(is.nan(cumprinc(0.09/12, 360, 125000, 10, 1, FALSE)))
})

test_that("nper matches Excel", {
  expect_equal(round(nper(0.12/12, -100, -1000, 10000, TRUE), 2), 59.67)
})

test_that("rate matches Excel and guards bad input", {
  expect_equal(round(rate(48, -200, 8000, 0, FALSE, 0.1) * 12, 4), 0.0924)
  expect_true(is.nan(rate(0, -200, 8000, 0, FALSE, 0.1)))  # nper = 0
})

test_that("ispmt matches Excel and guards bad input", {
  expect_equal(round(ispmt(0.10/12, 1, 36, 8000000), 2), -66666.67)
  expect_true(is.nan(ispmt(0.05,  0, 12, 10000)))  # per < 1
  expect_true(is.nan(ispmt(0.05, 99, 12, 10000)))  # per > nper
})
