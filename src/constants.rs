//! Contains useful constants and unit conversion functions

/// Tick rate, measured in Hz.
pub const TICK_RATE: f32 = 30.0;

/// The inverse of tick rate, measured in seconds.
// note: multiplying by this will often lead to worse precision than dividing by tick rate, but we
// have to do it to match tool.exe output
pub const TICK_RATE_INVERSE: f32 = 1.0 / TICK_RATE;

/// Approximate density of air in grams per milliliter (g/mL or g/cm³).
pub const AIR_DENSITY: f32 = 0.0011;

/// Approximate density of pure water in grams per milliliter (g/mL or g/cm³).
pub const WATER_DENSITY: f32 = 1.0;

/// Number of meters in one US foot.
pub const METERS_PER_FOOT: f32 = 0.3048;

/// Number of US feet in a world unit
pub const FEET_PER_WORLD_UNIT: f32 = 10.0;

/// Number of meters in one world unit.
///
/// Approximately equal to `3.048`.
// writing out `3.048` will net us a closer number, but it will result in different values than tool.exe
pub const METERS_PER_WORLD_UNIT: f32 = FEET_PER_WORLD_UNIT * METERS_PER_FOOT;

/// Number of world units per meter.
pub const WORLD_UNIT_PER_METER: f32 = 1.0 / METERS_PER_WORLD_UNIT;

/// Gravity in meters per second squared.
///
/// 9.78 = approximately 0.997 g (~0.997x Earth's gravity at sea level)
pub const GRAVITY_METERS_PER_SECOND_SQUARED: f32 = 9.78;

/// Gravity in world units per tick squared.
pub const GRAVITY_WORLD_UNITS_PER_TICK_SQUARED: f32 = const {
    // meters -> world units
    let gravity_world_units_per_second_squared = meters_to_world_units(GRAVITY_METERS_PER_SECOND_SQUARED);

    // seconds squared -> ticks squared
    let gravity_world_units_second_squared = per_seconds_squared_to_per_ticks_squared(gravity_world_units_per_second_squared);

    gravity_world_units_second_squared
};

/// Convert meters to world units.
#[inline]
pub const fn meters_to_world_units(meters: f32) -> f32 {
    meters * WORLD_UNIT_PER_METER
}

/// Convert world units to meters.
#[inline]
pub const fn world_units_to_meters(world_units: f32) -> f32 {
    world_units / WORLD_UNIT_PER_METER
}

/// Convert seconds to ticks.
#[inline]
pub const fn seconds_to_ticks(seconds: f32) -> f32 {
    seconds * TICK_RATE
}

/// Un-convert seconds to ticks.
///
/// This is the reverse of [`seconds_to_ticks`].
#[inline]
pub const fn reverse_seconds_to_ticks(ticks: f32) -> f32 {
    ticks / TICK_RATE
}

/// Convert ticks to seconds.
///
/// # Remarks
///
/// This is NOT the reverse of [`seconds_to_ticks`], as it multiplies by the reciprocal of tick rate
/// which can lead to different results due to floating point precision.
#[inline]
pub const fn ticks_to_seconds(seconds: f32) -> f32 {
    seconds * TICK_RATE_INVERSE
}

/// Un-convert ticks to seconds.
///
/// This is the reverse of [`ticks_to_seconds`].
#[inline]
pub const fn reverse_ticks_to_seconds(seconds: f32) -> f32 {
    seconds / TICK_RATE_INVERSE
}

/// Convert per seconds squared to per ticks squared.
#[inline]
pub const fn per_seconds_squared_to_per_ticks_squared(seconds_squared: f32) -> f32 {
    // the bits are just melting away... 🫠
    seconds_squared * TICK_RATE_INVERSE * TICK_RATE_INVERSE
}

/// Un-convert per seconds squared to per ticks squared.
///
/// This is the reverse of [`per_seconds_squared_to_per_ticks_squared`].
#[inline]
pub const fn reverse_per_seconds_squared_to_per_ticks_squared(ticks_squared: f32) -> f32 {
    ticks_squared / TICK_RATE_INVERSE / TICK_RATE_INVERSE
}

/// Length of a detail_object_collection cell in world units.
pub const DETAIL_OBJECT_WORLD_UNITS_PER_CELL: f32 = 8.0;

/// Size of an Xbox ADPCM chunk.
pub const XBOX_ADPCM_BLOCK_SIZE_BYTES: usize = 36;

/// Number of samples in an Xbox ADPCM block.
pub const XBOX_ADPCM_BLOCK_SAMPLE_COUNT: usize = 64;

/// Size of a 16-bit PCM sample in bytes.
pub const PCM_SAMPLE_SIZE_BYTES: usize = 2;
