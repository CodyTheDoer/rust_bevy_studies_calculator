pub trait FlexInput {
    fn to_f64(self) -> f64;
}

impl FlexInput for usize {
    fn to_f64(self) -> f64 {
        self as f64
    }
}
impl FlexInput for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

pub fn add<T: FlexInput, U: FlexInput>(left: T, right: U) -> f64 {
    let conv_left = left.to_f64();
    let conv_right = right.to_f64();

    conv_left + conv_right
}

pub fn subtract<T: FlexInput, U: FlexInput>(left: T, right: U) -> f64 {
    let conv_left = left.to_f64();
    let conv_right = right.to_f64();

    conv_left - conv_right
}

pub fn multiply<T: FlexInput, U: FlexInput>(left: T, right: U) -> f64 {
    let conv_left = left.to_f64();
    let conv_right = right.to_f64();

    conv_left * conv_right
}

pub fn divide<T: FlexInput, U: FlexInput>(left: T, right: U) -> f64 {
    let conv_left = left.to_f64();
    let conv_right = right.to_f64();

    conv_left / conv_right
}

#[cfg(test)]
mod calc_backend_functionality {
    use super::*;

    #[test]
    fn add_check() {
        assert_eq!(add(2, 2), 4.0);
        assert_eq!(add(12.0, 12), 24.0);
        assert_eq!(add(24.0, 2.0), 26.0);
    }

    #[test]
    fn subtract_check() {
        assert_eq!(subtract(2, 2), 0.0);
        assert_eq!(subtract(12.0, 12), 0.0);
        assert_eq!(subtract(24.0, 2.0), 22.0);
    }

    #[test]
    fn multiply_check() {
        assert_eq!(multiply(2, 2), 4.0);
        assert_eq!(multiply(12.0, 12), 144.0);
        assert_eq!(multiply(24.0, 2.0), 48.0);
    }

    #[test]
    fn divide_check() {
        assert_eq!(divide(2, 2), 1.0);
        assert_eq!(divide(12.0, 12), 1.0);
        assert_eq!(divide(24, 2.0), 12.0);
    }
}