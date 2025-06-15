#![allow(unused)]

macro_rules! assert_similar {
    ($a: expr, $b: expr, $max: expr) => {{
        use $crate::float::FloatOps;

        let a_real = $a as f64;
        let b_real = $b as f64;
        let max_real = $max as f64;

        let a = a_real.abs();
        let b = b_real.abs();
        let c = (a - b).abs();

        if c > $max {
            panic!("assert_similar failed (a={a_real}, b={b_real}, max={max_real}, actual={c})")
        }
    }};
}
pub(crate) use assert_similar;
