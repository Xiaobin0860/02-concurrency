use std::{
    fmt::{self, Display},
    ops::{Add, Deref, Mul},
};

use anyhow::{bail, Result};

pub struct Vector<T> {
    data: Vec<T>,
}

impl<T> Vector<T> {
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        Self { data: data.into() }
    }
}

impl<T> Deref for Vector<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Display for Vector<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (i, x) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", x)?;
        }
        write!(f, "}}")
    }
}

pub fn dot_product<T>(a: &Vector<T>, b: &Vector<T>) -> Result<T>
where
    T: Mul<Output = T> + Add<Output = T> + Copy + Default,
{
    if a.len() != b.len() {
        bail!(
            "invalid vector size: a.len-{} != b.len-{}",
            a.len(),
            b.len()
        );
    }

    Ok(a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| x * y)
        .fold(T::default(), |acc, x| acc + x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product() {
        let a = Vector {
            data: vec![1, 2, 3],
        };
        let b = Vector {
            data: vec![4, 5, 6],
        };
        assert_eq!(32, dot_product(&a, &b).unwrap());
    }
}
