pub mod bees;
pub mod func;
pub mod utils;

use bees::FoodResources;
use func::Func;
use ndarray::Array1;
use ndarray_rand::rand_distr::Uniform;

fn f(arr: &Array1<f64>) -> f64 {
    let x = arr[0];
    let y = arr[1];

    (x + 2.0 * y - 7.0).powi(2) + (2.0 * x + y - 5.0).powi(2)
}

fn main() {
    let iters = 3;
    let size = 3;
    let dim = 2;
    let limit = dim * 3;
    let distribution = Uniform::new(-5.0, 5.0);
    let func = Func::new(f);

    let mut resources = FoodResources::new(func, size, dim, limit, distribution);

    println!("# Fuentes de alimento iniciales");
    println!("{}", resources.summary());

    for i in 0..iters {
        println!("# Iteración {}", i + 1);
        resources.step();
    }
}
