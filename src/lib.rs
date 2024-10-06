fn into_float(arg: usize) -> f64 {
    let converted_float = arg as f64;
    converted_float
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn subtract(left: usize, right: usize) -> usize {
    left - right
}

pub fn multiply(left: usize, right: usize) -> usize {
    left * right
}

pub fn divide(left: usize, right: usize) -> f64 {
    let conv_left = into_float(left);
    let conv_right = into_float(right);

    conv_left / conv_right
}

#[cfg(test)]
mod calc_backend_functionality {
    use super::*;

    #[test]
    fn add_check() {
        let result = add(2, 2);
        assert_eq!(result, 4);

        let result = add(12, 12);
        assert_eq!(result, 24);

        let result = add(24, 2);
        assert_eq!(result, 26);
    }

    #[test]
    fn subtract_check() {
        let result = subtract(2, 2);
        assert_eq!(result, 0);
        
        let result = subtract(12, 12);
        assert_eq!(result, 0);
        
        let result = subtract(24, 2);
        assert_eq!(result, 22);
    }

    #[test]
    fn multiply_check() {
        let result = multiply(2, 2);
        assert_eq!(result, 4);

        let result = multiply(12, 12);
        assert_eq!(result, 144);

        let result = multiply(24, 2);
        assert_eq!(result, 48);
    }

    #[test]
    fn divide_check() {
        let result = divide(2, 2);
        assert_eq!(result, 1.0);
        
        let result = divide(12, 12);
        assert_eq!(result, 1.0);
        
        let result = divide(24, 2);
        assert_eq!(result, 12.0);
        
        // let result = divide(22.5, 2);
        // assert_eq!(result, 11.25);
    }
}