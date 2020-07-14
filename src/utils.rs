use rand::distributions::uniform::SampleUniform;
use rand::Rng;

pub trait MoreRandom {
    fn gen_range_except<T>(&mut self, lo: T, hi: T, except: T) -> T
    where
        T: SampleUniform + PartialEq + Copy;
}

impl<R> MoreRandom for R
where
    R: Rng,
{
    fn gen_range_except<T>(&mut self, lo: T, hi: T, except: T) -> T
    where
        T: SampleUniform + PartialEq + Copy,
    {
        let mut rand = self.gen_range(lo, hi);
        while rand == except {
            rand = self.gen_range(lo, hi);
        }

        rand
    }
}

pub fn choose_probs(rand: f64, probs: &[f64]) -> usize {
    let mut it = probs.iter().enumerate();
    let mut acc = 0.0;
    let (mut idx, prob) = it.next().unwrap();
    acc += prob;

    while acc < rand {
        let (i, prob) = it.next().unwrap();
        acc += prob;
        idx = i;
    }

    idx
}
