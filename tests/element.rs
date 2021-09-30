use redc::element::Element;
use redc::Redc;
#[test]
fn test_u64() {
    let factor = (1u64 << 32) - 5;
    let increment = u64::MAX - 60;
    let modulus = u64::MAX - 58;
    {
        let mut x = factor as u128;

        let f = modulus.setup_field();
        let mut n = f.wrap_element(factor);
        let wrapped_increment = f.wrap_element(increment);

        for _ in 0..1000 {
            x *= x;
            x += increment as u128;
            x %= modulus as u128;

            n = n * n;
            n = n + wrapped_increment;

            assert_eq!(x as u64, n.to_normal());
        }
    }
}
