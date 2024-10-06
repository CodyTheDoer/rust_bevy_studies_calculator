// Setup ToF64 as a custom trait
trait ToF64 {
    fn to_f64(self) -> f64;
}

// implement trait for usize or f64 user entries
impl ToF64 for usize {
    fn to_f64(self) -> f64 {
        self as f64
    }
}
impl ToF64 for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

// Now our convert_float function can accept any type that implements ToF64
fn convert_float<T: ToF64>(value: T) -> f64 {
    value.to_f64()
}

// Public facing functions for calc use and impleted user input generics for backend use.
pub fn add<T: ToF64, U: ToF64>(left: T, right: U) -> f64 {
    let conv_left = convert_float(left);
    let conv_right = convert_float(right);

    conv_left + conv_right
}

pub fn subtract<T: ToF64, U: ToF64>(left: T, right: U) -> f64 {
    let conv_left = convert_float(left);
    let conv_right = convert_float(right);

    conv_left - conv_right
}

pub fn multiply<T: ToF64, U: ToF64>(left: T, right: U) -> f64 {
    let conv_left = convert_float(left);
    let conv_right = convert_float(right);

    conv_left * conv_right
}

pub fn divide<T: ToF64, U: ToF64>(left: T, right: U) -> f64 {
    let conv_left = convert_float(left);
    let conv_right = convert_float(right);

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