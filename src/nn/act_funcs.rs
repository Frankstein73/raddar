use std::f64::consts::PI;

use tch::Tensor;

use crate::nn::{Module, NonParameterModule};

#[derive(Debug)]
pub struct GeLU;

#[derive(Debug)]
pub struct LeakyReLU {
    lambda: f64,
}
impl LeakyReLU {
    pub fn new(lambda: f64) -> LeakyReLU {
        LeakyReLU { lambda: lambda }
    }
}
impl NonParameterModule for GeLU {}
impl NonParameterModule for LeakyReLU {}
impl Module for GeLU {
    fn forward(&self, input: &Tensor) -> Tensor {
        let z = (input + &input.pow_tensor_scalar(3) * 0.044715) * (2.0f64 / PI).sqrt();
        0.5 * input * (1 + z.tanh())
    }
}
impl Module for LeakyReLU {
    fn forward(&self, input: &Tensor) -> Tensor {
        let y = -input * self.lambda;
        let condition = input.ge(0);
        input.where_self(&condition, &y)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gelu_test() {
        let input = Tensor::of_slice2(&[[1.0], [3.0], [5.0], [4.0], [8.0], [10.0], [2.0], [6.0]]);
        let output = GeLU.forward(&input);
        let expected = Tensor::of_slice2(&[
            [0.8413],
            [2.9960],
            [5.0000],
            [3.9999],
            [8.0000],
            [10.0000],
            [1.9545],
            [6.0000],
        ]);
        assert!(f64::from((output - expected).square().sum(tch::Kind::Double)) < 1e-4);
    }
}
