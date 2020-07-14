use crate::{
    func::Func,
    utils::{choose_probs, MoreRandom},
};
use ndarray::Array1;
use ndarray_rand::rand_distr::Distribution;
use ndarray_rand::RandomExt;
use prettytable::{cell, format::consts as formats, row, table};
use rand::{thread_rng, Rng};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct FoodResource {
    raw: Array1<f64>,
    count: usize,
}

impl FoodResource {
    pub fn new(dim: usize, distribution: impl Distribution<f64>) -> Self {
        let raw = Array1::random((dim,), distribution);

        Self { raw, count: 0 }
    }

    pub fn reset(&mut self, distribution: impl Distribution<f64>) {
        let dim = self.raw.len();
        *self = Self::new(dim, distribution);
    }

    pub fn reset_count(&mut self) {
        self.count = 0
    }

    pub fn inc_count(&mut self) {
        self.count += 1;
    }

    pub fn fitness(&self, func: &Func<Array1<f64>>) -> (f64, f64) {
        let fnx = func.calculate(&self.raw);
        let fit = if fnx >= 0.0 {
            1.0 / (fnx + 1.0)
        } else {
            1.0 + fnx.abs()
        };

        (fnx, fit)
    }
}

impl Index<usize> for FoodResource {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        self.raw.index(index)
    }
}

impl IndexMut<usize> for FoodResource {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.raw.index_mut(index)
    }
}

pub struct FoodResources<'a, D>
where
    D: Distribution<f64>,
{
    func: Func<'a, Array1<f64>>,
    best: FoodResource,
    resources: Vec<FoodResource>,
    dim: usize,
    limit: usize,
    distribution: D,
}

impl<'a, D> FoodResources<'a, D>
where
    D: Distribution<f64>,
{
    pub fn new(
        func: Func<'a, Array1<f64>>,
        size: usize,
        dim: usize,
        limit: usize,
        distribution: D,
    ) -> Self {
        let resources: Vec<_> = (0..size)
            .map(|_| FoodResource::new(dim, &distribution))
            .collect();

        let best = resources
            .iter()
            .max_by(|a, b| {
                let (_, fit_a) = a.fitness(&func);
                let (_, fit_b) = b.fitness(&func);

                fit_a.partial_cmp(&fit_b).expect("Got a NaN, check fitness")
            })
            .expect("Population is empty")
            .clone();

        Self {
            func,
            best,
            resources,
            dim,
            limit,
            distribution,
        }
    }

    pub fn step(&mut self) {
        // Replace with new resources
        println!("## Enviar abejas empleadas (soluciones candidatas)");
        self.employed_bees_step();

        println!("## Calcular la probabilidad de selección de cada fuente");
        let probs = self.calculate_probs();

        println!("## Enviar abejas observadoras");
        self.onlooker_bees_step(probs);

        println!("## Enviar abejas exploradoras");
        self.explorer_bees_step();
        println!("{}", self.summary());
    }

    pub fn calculate_probs(&self) -> Vec<f64> {
        let mut table = table!();
        table.set_format(*formats::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row![
            "Fuente",
            "x",
            "f(x)",
            "fit",
            "prob",
            "prob_acum",
            "Cont."
        ]);

        let sum = self
            .resources
            .iter()
            .map(|r| r.fitness(&self.func).1)
            .fold(0.0, |acc, qty| acc + qty);

        let mut probs = Vec::new();
        let mut acc = 0.0;
        for (i, resource) in self.resources.iter().enumerate() {
            let (fnx, fit) = resource.fitness(&self.func);

            let prob = fit / sum;
            acc += prob;

            probs.push(prob);
            let count = resource.count;

            table.add_row(row![i + 1, resource.raw, fnx, fit, prob, acc, count]);
        }

        println!("{}", table);
        probs
    }

    pub fn employed_bees_step(&mut self) {
        let mut table = table!();
        table.set_format(*formats::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row![
            "Fuente orig.",
            "k",
            "j",
            "phi",
            "v",
            "f(v)",
            "fit",
            "Mejora?",
            "Cont."
        ]);

        let size = self.resources.len();
        let mut rng = thread_rng();

        let mut new_resources = Vec::new();
        for (i, resource) in self.resources.iter().enumerate() {
            let phi = rng.gen_range(-1.0, 1.0);
            let k = rng.gen_range_except(0, size, i);
            let j = rng.gen_range(0, self.dim);

            // Randomly "mutate" the original resource
            let mut new_resource = resource.clone();
            new_resource[j] = resource[j] + phi * (resource[j] - self.resources[k][j]);

            let (_, orig_fit) = resource.fitness(&self.func);
            let (near_fnx, near_fit) = new_resource.fitness(&self.func);

            // Check which one is better, the new or the old one?
            let improve = near_fit > orig_fit;
            let improve_msg = if improve { "Sí" } else { "No" };
            let count_msg = if improve { 0 } else { resource.count + 1 };

            table.add_row(row![
                i + 1,
                k + 1,
                j,
                phi,
                &new_resource.raw,
                near_fnx,
                near_fit,
                improve_msg,
                count_msg
            ]);

            if improve {
                new_resource.reset_count();
                new_resources.push(new_resource);
            } else {
                let mut resource = resource.clone();
                resource.inc_count();
                new_resources.push(resource);
            }
        }

        println!("{}", table);
        self.resources = new_resources;
    }

    pub fn onlooker_bees_step(&mut self, mut probs: Vec<f64>) {
        let mut table = table!();
        table.set_format(*formats::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row![
            "i", "k", "j", "phi", "v", "f(v)", "fit", "Mejora?", "Cont."
        ]);

        let size = self.resources.len();
        let mut rng = thread_rng();

        for b in 0..size {
            println!("### Abeja observadora #{}", b + 1);
            let rand = rng.gen_range(0.0, 1.0);
            println!("### Aleatorio: {}", rand);

            let i = choose_probs(rand, &probs);
            let k = rng.gen_range_except(0, size, i);
            let j = rng.gen_range(0, self.dim);

            let phi = rng.gen_range(-1.0, 1.0);

            // Randomly "mutate" the original resource
            let resource = self.resources[i].clone();
            let mut new_resource = self.resources[i].clone();
            new_resource[j] = resource[j] + phi * (resource[j] - self.resources[k][j]);

            let (_, orig_fit) = resource.fitness(&self.func);
            let (near_fnx, near_fit) = new_resource.fitness(&self.func);

            // Check which one is better, the new or the old one?
            let improve = near_fit > orig_fit;
            let improve_msg = if improve { "Sí" } else { "No" };
            let count_msg = if improve { 0 } else { resource.count + 1 };

            table.add_row(row![
                i + 1,
                k + 1,
                j,
                phi,
                &new_resource.raw,
                near_fnx,
                near_fit,
                improve_msg,
                count_msg
            ]);

            println!("{}", table);
            table.remove_row(0);

            if improve {
                new_resource.reset_count();
                self.resources[i] = new_resource;
            } else {
                self.resources[i].inc_count();
            }

            println!("### Nuevas probabilidades");
            probs = self.calculate_probs();
        }
    }

    pub fn explorer_bees_step(&mut self) {
        for resource in &mut self.resources {
            if resource.count >= self.limit {
                resource.reset(&self.distribution);
            }

            let (_, this_fit) = resource.fitness(&self.func);
            let (_, best_fit) = self.best.fitness(&self.func);

            if this_fit > best_fit {
                self.best = resource.clone();
            }
        }
    }

    pub fn summary(&self) -> String {
        let mut table = table!();
        table.set_format(*formats::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row!["Fuente", "x", "f(x)", "fit", "Cont."]);

        for (i, resource) in self.resources.iter().enumerate() {
            let (fnx, fit) = resource.fitness(&self.func);
            let count = resource.count;
            table.add_row(row![i + 1, resource.raw, fnx, fit, count]);
        }

        let (fnx, fit) = self.best.fitness(&self.func);
        table.add_empty_row();
        table.add_row(row!["Mejor", &self.best.raw, fnx, fit, "---"]);

        table.to_string()
    }
}
