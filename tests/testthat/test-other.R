test_that("depreciation functions match Excel", {
  expect_equal(sln(30000, 7500, 10), 2250)
  expect_true(is.nan(sln(30000, 7500, 0)))
  expect_equal(round(syd(30000, 7500, 10, 1), 2), 4090.91)
  expect_equal(round(db(1000000, 100000, 6, 1, 7), 2), 186083.33)
  expect_equal(ddb(2400, 300, 10, 1, 2), 480)
  expect_equal(round(vdb(2400, 300, 10, 0, 1, 2, FALSE), 2), 480)
  expect_equal(round(amordegrc(2400, 39679, 39813, 300, 1, 0.15, 1), 2), 776)
  expect_equal(round(amorlinc(2400, 39679, 39813, 300, 1, 0.15, 1), 2), 360)
})

test_that("coupon date functions match Excel", {
  expect_equal(coupdaybs("2011-01-25", "2011-11-15", 2, 1), 71)
  expect_equal(coupdays("2011-01-25", "2011-11-15", 2, 1), 181)
  expect_equal(coupdaysnc("2011-01-25", "2011-11-15", 2, 1), 110)
  expect_equal(coupnum("2007-01-25", "2008-11-15", 2, 1), 4)
  # Guard: settlement >= maturity
  expect_true(is.nan(coupdaybs("2025-01-01", "2020-01-01", 2, 1)))
  expect_true(is.nan(coupdays("2020-01-01", "2020-01-01", 2, 0)))
})

test_that("bond functions match Excel", {
  expect_equal(round(price("2008-02-15","2017-11-15",0.0575,0.065,100,2,0), 2), 94.63)
  expect_equal(round(yield_("2008-02-15","2016-11-15",0.0575,95.04287,100,2,0), 4), 0.065)
  expect_equal(round(yieldmat("2008-03-15","2008-11-03","2007-11-08",0.0625,100.0123,0), 4), 0.0610)
  expect_equal(round(pricedisc("2008-02-16","2008-03-01",0.0525,100,2), 3), 99.795)
  expect_equal(round(yielddisc("2008-02-16","2008-03-01",99.795,100,2), 4), 0.0525)
  expect_equal(round(duration("2008-01-01","2016-01-01",0.08,0.09,2,1), 4), 5.9938)
  expect_equal(round(mduration("2008-01-01","2016-01-01",0.08,0.09,2,1), 4), 5.7355)
  expect_equal(round(accrintm("2008-04-01","2008-06-15",0.10,1000,3), 2), 20.54)
  expect_equal(round(oddfprice("2008-11-11","2021-03-01","2008-10-15","2009-03-01",0.0785,0.0625,100,2,1), 2), 113.60)
  expect_equal(round(oddlprice("2008-02-07","2008-06-15","2007-10-15",0.0375,0.0405,100,2,0), 2), 99.88)
  # Guard: settlement >= maturity
  expect_true(is.nan(price("2030-01-01","2020-01-01",0.05,0.06,100,2,0)))
})

test_that("misc functions match Excel", {
  expect_equal(round(effect(0.0525, 4), 6), 0.053543)
  expect_true(is.nan(effect(0.05, 0)))
  expect_equal(round(nominal(0.053543, 4), 4), 0.0525)
  expect_equal(dollarde(1.02, 16), 1.125)
  expect_equal(dollarfr(1.125, 16), 1.02)
  expect_equal(round(fvschedule(1, c(0.09, 0.11, 0.10)), 4), 1.3309)
  expect_true(is.nan(fvschedule(1, numeric(0))))
  expect_equal(round(pduration(0.025, 2000, 2200), 4), 3.8599)
  expect_equal(round(tbillprice("2008-03-31","2008-06-01",0.09), 2), 98.45)
  expect_equal(round(tbillyield("2008-03-31","2008-06-01",98.45), 4), 0.0914)
  expect_equal(round(tbilleq("2008-03-31","2008-06-01",0.0914), 4), 0.0942)
})
