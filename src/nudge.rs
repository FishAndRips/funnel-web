//! Contains the [`fix_decimal_rounding`] function.

use core::fmt::Write;

const MAX_SIG_FIGS: usize = 6;

/// Nudges a float, rounding it to a nicer-looking number.
///
/// This is used for numbers written by a human that was transformed by some sort of postprocessing
/// algorithm.
///
/// Example:
///
/// ```
/// use funnel_web::nudge::fix_decimal_rounding;
///
/// assert_eq!(fix_decimal_rounding(1.75000035762786865234375), 1.75);
/// assert_eq!(fix_decimal_rounding(1.74999964237213134765625), 1.75);
/// assert_eq!(fix_decimal_rounding(999.99994), 1000.0);
/// ```
pub fn fix_decimal_rounding(input: f32) -> f32 {
    if !input.is_finite() {
        return input
    }

    let signum = input.signum();
    let abs = input.abs();

    let mut buf = NudgeBuffer::default();
    core::fmt::write(&mut buf, format_args!("{abs}")).expect("can't write float to buffer");

    let str_index = buf.offset;
    let mut buf = buf.buffer;

    let written = &mut buf[..str_index];

    // First, find the most significant digits
    if written.len() <= MAX_SIG_FIGS {
        return input;
    }

    macro_rules! ignore_dot_iter {
        ($a:expr) => {{
            let mut first_sig_fig_found = false;
            ($a)
                .iter_mut()
                .enumerate()
                .filter_map(move |(i,b)| {
                    if *b == b'.' || (!first_sig_fig_found && *b == b'0') {
                        return None;
                    }
                    first_sig_fig_found = true;
                    Some((b,i))
                })
        }};
    }

    let mut sig_figs_end = str_index;
    let mut sig_figs_index = 0usize;
    let mut round_up = false;

    for (byte, index) in ignore_dot_iter!(written) {
        if sig_figs_index == MAX_SIG_FIGS {
            sig_figs_end = index;
            round_up = *byte >= b'5';
        }
        if sig_figs_index >= MAX_SIG_FIGS {
            *byte = b'0';
        }
        sig_figs_index += 1;
    }

    // Do rounding here
    let mut prepend_one = false;
    if round_up {
        for (byte, index) in ignore_dot_iter!(written[..sig_figs_end]).rev() {
            if *byte == b'9' {
                if index == 0 {
                    prepend_one = true;
                }
                *byte = b'0';
            } else {
                *byte += 1;
                break;
            }
        }
    }

    debug_assert!(!prepend_one, "should be no more to prepend");
    let fstr = core::str::from_utf8(&written).expect("should be utf-8");

    let f: f64 = fstr.parse().map_err(|e| panic!("can't parse the float we just made `{fstr}` as a float: {e:?}")).unwrap();
    (f as f32) * signum
}

struct NudgeBuffer {
    buffer: [u8; 512], // 512 digits should be more than enough to hold any 64-bit float
    offset: usize
}

impl Write for NudgeBuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s_bytes = s.as_bytes();
        self.buffer[self.offset..s_bytes.len() + self.offset].copy_from_slice(s_bytes);
        self.offset += s_bytes.len();

        Ok(())
    }
}

impl Default for NudgeBuffer {
    fn default() -> Self {
        Self {
            buffer: [b'0'; 512],
            offset: 1 // leave a byte to carry a 1
        }
    }
}

#[cfg(test)]
mod test {
    use crate::nudge::fix_decimal_rounding;

    #[test]
    pub fn test_nudgification() {
        fn test_nudge(expected: f32, value_to_nudge: f32) {
            assert_ne!(expected, value_to_nudge, "values already equal (FP precision weirdness?)");
            assert_eq!(expected, fix_decimal_rounding(value_to_nudge), "nudging failed");
        }
        fn test_no_nudge(value_to_nudge: f32) {
            assert_eq!(value_to_nudge, fix_decimal_rounding(value_to_nudge), "nudged when it shouldn't");
        }

        test_nudge(1.75, 1.75000035762786865234375);
        test_nudge(1.75, 1.74999964237213134765625);
        test_nudge(1000.0, 999.99994);

        test_nudge(0.0005, 0.000500000081956386566162109375);
        test_nudge(0.0005, 0.00049999985);
        test_nudge(0.001, 0.0009999999);
        test_nudge(0.0100098, 0.010009766);
        test_nudge(-0.0100098, -0.010009766);

        test_nudge(1.0, 0.9999995);
        test_nudge(1.0, 1.0000003);

        test_no_nudge(33.3333);
        test_no_nudge(1.0);
        test_no_nudge(1.75);
        test_no_nudge(0.0100098);
        test_no_nudge(0.0100098);
        test_nudge(33.3333, 33.333332061767578125);
        test_nudge(-33.3333, -33.333332061767578125);
    }
}
