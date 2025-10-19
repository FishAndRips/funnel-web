#[cfg(test)]
macro_rules! assert_similar {
    ($a: expr, $b: expr, $max: expr) => {{
        use $crate::float::FloatOps;

        let a_real = $a as f32;
        let b_real = $b as f32;
        let max_real = $max as f32;

        let difference = (a_real - b_real).fw_fabs();

        if difference > max_real {
            panic!("assert_similar failed (a={a_real}, b={b_real}, max={max_real}, actual={difference})")
        }
    }};
}

#[cfg(test)]
pub(crate) use assert_similar;
