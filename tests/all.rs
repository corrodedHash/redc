use twoword::TwoWord;

use redc::Field;

use redc::Redc;

#[test]
fn test_redc() {
    let field = 23u64.setup_field();
    let a_original = 7u64;
    let b_original = 17u64;
    let a = a_original.to_montgomery(&field);
    let b = b_original.to_montgomery(&field);
    let r = field.redc(u128::from(a) * u128::from(b));
    assert_eq!(r.to_normal(&field), 4);
}

#[test]
fn test_redc_overflow() {
    let field = u64::MAX.setup_field();
    // primes 2**64 - 59, 2**64 - 83
    let a_original = u64::MAX - 58;
    let b_original = u64::MAX - 82;
    let a = a_original.to_montgomery(&field);
    assert_eq!(a, a_original);
    let b = b_original.to_montgomery(&field);
    assert_eq!(b, b_original);
    let r = field.redc(u128::from(a) * u128::from(b));
    assert_eq!(r, 4756);
    assert_eq!(r.to_normal(&field), 4756);
}

#[test]
fn test_redc_overflow_better() {
    let field = (u64::MAX - 2).setup_field();
    // primes 2**64 - 59, 2**64 - 83
    let a_original = u64::MAX - 58;
    let b_original = u64::MAX - 82;
    let a = a_original.to_montgomery(&field);
    assert_eq!(a, 18_446_744_073_709_551_445);
    let b = b_original.to_montgomery(&field);
    assert_eq!(b, 18_446_744_073_709_551_373);
    let r = field.redc(u128::from(a) * u128::from(b));
    assert_eq!(r, 13440);
    assert_eq!(r.to_normal(&field), 4480);
}

#[test]
fn test_redc_u128() {
    let field = 23u128.setup_field();
    let a_original = 7u128;
    let b_original = 17u128;
    let a = a_original.to_montgomery(&field);
    let b = b_original.to_montgomery(&field);
    let r = field.redc(TwoWord::<u128>::mult(a, b));
    assert_eq!(r.to_normal(&field), 4);
}

#[test]
fn test_redc_overflow_u128() {
    let field = u128::MAX.setup_field();
    // primes 2**128 - 159, 173
    let a_original = u128::MAX - 58;
    let b_original = u128::MAX - 82;
    let a = a_original.to_montgomery(&field);
    assert_eq!(a, a_original);
    let b = b_original.to_montgomery(&field);
    assert_eq!(b, b_original);
    let r = field.redc(TwoWord::mult(a, b));
    assert_eq!(r, 4756);
    assert_eq!(r.to_normal(&field), 4756);
}

#[test]
fn test_redc_overflow_better_u128() {
    let field = (u128::MAX - 2).setup_field();
    // primes 2**128 - 159, 173
    let a_original = u128::MAX - 158;
    let b_original = u128::MAX - 172;
    let a = a_original.to_montgomery(&field);
    assert_eq!(a, 340_282_366_920_938_463_463_374_607_431_768_210_985);
    let b = b_original.to_montgomery(&field);
    assert_eq!(b, 340_282_366_920_938_463_463_374_607_431_768_210_943);
    let r = field.redc(TwoWord::mult(a, b));
    assert_eq!(r, 79560);
    assert_eq!(r.to_normal(&field), 26520);
}

#[test]
fn test_redc_rug() {
    let field = rug::Integer::from(23).setup_field();
    let a_original = rug::Integer::from(7);
    let b_original = rug::Integer::from(17);
    let a = a_original.to_montgomery(&field);
    assert_eq!(a, 17);
    let b = b_original.to_montgomery(&field);
    assert_eq!(b, 15);
    let r = field.redc(a * b);
    assert_eq!(r.to_normal(&field), 4);
}
