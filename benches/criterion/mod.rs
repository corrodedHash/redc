use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use redc::element::Element;
use redc::Field;
use redc::Redc;

fn redc(c: &mut Criterion) {
    let mut group = c.benchmark_group("redc");

    group.plot_config(
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic),
    );
    let field = 9241u64.setup_field();
    for i in [(1_u128 << 32) - 5] {
        group.bench_with_input(BenchmarkId::from_parameter(i), &i, |b, i| {
            b.iter(|| field.redc(*i))
        });
    }
    group.finish();
}

fn multiplication(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiplication");

    group.plot_config(
        criterion::PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic),
    );
    let factor = (1u64 << 32) - 5;
    let increment = 1u64;
    let modulus = 9241;
    for i in [1, 10, 100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("normal", i), &i, |b, loop_count| {
            b.iter(|| {
                let mut x = factor;
                for _ in 0..*loop_count {
                    x *= x;
                    x += increment;
                    x %= modulus;
                }
                x
            })
        });
        group.bench_with_input(BenchmarkId::new("raw_redc", i), &i, |b, loop_count| {
            b.iter(|| {
                let f = modulus.setup_field();
                let mut n = factor.to_montgomery(&f);
                let i = increment.to_montgomery(&f);
                for _ in 0..*loop_count {
                    n = f.redc(n as u128 * n as u128);
                    n += i;
                    if n >= modulus {
                        n -= modulus;
                    }
                }
                f.redc(n as u128)
            })
        });
        group.bench_with_input(BenchmarkId::new("wrapped_redc", i), &i, |b, loop_count| {
            b.iter(|| {
                let f = modulus.setup_field();
                let mut n = f.wrap_element(factor);
                let wrapped_increment = f.wrap_element(increment);
                for _ in 0..*loop_count {
                    n = n * n;
                    n = n + wrapped_increment;
                }
                n.to_normal()
            })
        });
    }
    group.finish();
}

criterion_group!(name = benches; config = Criterion::default().measurement_time(std::time::Duration::from_millis(100)).warm_up_time(std::time::Duration::from_millis(5)); targets = multiplication, redc);
criterion_main!(benches);
