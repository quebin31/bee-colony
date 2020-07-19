pub mod bees;
pub mod func;
pub mod utils;

use bees::FoodResources;
use func::Func;
use ndarray::Array1;
use ndarray_rand::rand_distr::Uniform;
use prettytable::{cell, format::consts as formats, row, table};

fn f(arr: &Array1<f64>) -> f64 {
    let x = arr[0];
    let y = arr[1];

    (x + 2.0 * y - 7.0).powi(2) + (2.0 * x + y - 5.0).powi(2)
}

fn main() {
    let iters = 200;
    let size = 3;
    let dim = 2;
    let limit = dim * 3;
    let distribution = Uniform::new(-10.0, 10.0);
    let func = Func::new(f);

    let mut table = table! {
        ["# iteraciones", iters],
        ["Dimensión", dim],
        ["SN", size],
        ["Limit", limit],
        ["Distr", format!("{:?}", (-10.0, 10.0))]
    };

    table.set_titles(row!["Parámetro", "Valor"]);
    table.set_format(*formats::FORMAT_NO_LINESEP_WITH_TITLE);
    println!("{}", table);

    let mut resources = FoodResources::new(func, size, dim, limit, distribution);

    println!("# Fuentes de alimento iniciales");
    println!("{}", resources.summary());

    for i in 0..iters {
        println!("# Iteración {}", i + 1);
        resources.step();
    }
}
