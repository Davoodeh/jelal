from jelal import *

# Create an ordinal directly
ordinal = 1 * 31 + 13
# or using monthday
monthday = MonthDay(2, 13)
# function equals are available (discouraged)
assert monthday.ext_cmp(_monthday_new(2, 13)) == 0

ordinal_from_monthday = monthday.to_ordinal()
assert ordinal != ordinal_from_monthday

# Give to create a date
fixed_point = Date(1404, ordinal)
# Use methods on it, for example add days
expected_moved = Date(1404, ordinal + 11)
moved = fixed_point.add_days(11)
assert expected_moved.ext_cmp(moved) == 0

