pub const MIN: f32 = 0.5;
pub const MAX: f32 = 3.0;
pub const STEP: f32 = 0.25;

pub fn clamp(value: f32) -> f32 {
    value.clamp(MIN, MAX)
}

pub fn step_in(value: f32) -> f32 {
    clamp(value + STEP)
}

pub fn step_out(value: f32) -> f32 {
    clamp(value - STEP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_bounds() {
        assert_eq!(clamp(0.1), MIN);
        assert_eq!(clamp(9.0), MAX);
        assert_eq!(clamp(1.3), 1.3);
    }

    #[test]
    fn steps_within_bounds() {
        assert_eq!(step_in(2.9), MAX);
        assert_eq!(step_out(0.6), MIN);
        assert_eq!(step_in(1.0), 1.25);
    }
}
