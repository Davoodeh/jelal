// compile with LD_LIBRARY_PATH set and `gcc file.c -ljelal`
// TODO make a Makefile for these
#include <jelal.h>

int main() {
    // Create an ordinal directly
    // if using CFFI:
    /* UOrdinal ordinal = 1 * MONTHDAY_MAX_DAY + 13; // IF CFFI */
    UOrdinal ordinal = 1 * MonthDay_MAX_DAY + 13; // IF CBINDGEN
    // or using monthday
    MonthDay monthday = monthday_new(2, 13);
    Ordinal ordinal_from_monthday = monthday_to_ordinal(&monthday);
    int has_error = ordinal != ordinal_from_monthday;

    // Give to create a date
    Date fixed_point = date_new(1404, ordinal);
    // Use methods on it, for example add days
    Date expected_moved = date_new(1404, ordinal + 11);
    Date moved = date_add_days(fixed_point, 11);
    has_error = has_error || (date_ext_cmp(&expected_moved, &moved) != 0);

    return has_error;
}
