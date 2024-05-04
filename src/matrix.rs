use crate::{dot_product, Vector};
use anyhow::{bail, Result};
use std::{
    fmt::{self, Debug, Display},
    ops::{Add, Mul},
    sync::mpsc,
    thread,
};

// 1. matrix multiplication and unit tests
// 2. thread one-shot channel

const THREAD_COUNT: usize = 4;

pub struct Matrix<T> {
    data: Vec<T>,
    row: usize,
    col: usize,
}

impl<T> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}

struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

struct MsgOutput<T> {
    idx: usize,
    val: T,
}

struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>,
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Mul<Output = T> + Add<Output = T> + Copy + Default + Send + 'static,
{
    if a.col != b.row {
        bail!("invalid matrix size: a.col-{} != b.row-{}", a.col, b.row);
    }

    let senders = (0..THREAD_COUNT)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                for msg in rx {
                    let val = dot_product(&msg.input.row, &msg.input.col)?;
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        val,
                    }) {
                        eprintln!("send error: {e:?}");
                    }
                }
                Ok::<_, anyhow::Error>(())
            });
            tx
        })
        .collect::<Vec<_>>();

    // a row vector dot b col vector
    let matrix_len = a.row * b.col;
    let mut receivers = Vec::with_capacity(matrix_len);
    for i in 0..a.row {
        for j in 0..b.col {
            let a_row_vector = Vector::new(&a.data[i * a.col..(i + 1) * a.col]);
            let b_col_data = b.data[j..]
                .iter()
                .step_by(b.col)
                .copied()
                .collect::<Vec<_>>();
            let b_col_vector = Vector::new(b_col_data);
            let matrix_idx = i * b.col + j;
            let (tx, rx) = oneshot::channel();
            senders[matrix_idx % THREAD_COUNT]
                .send(Msg {
                    input: MsgInput {
                        idx: matrix_idx,
                        row: a_row_vector,
                        col: b_col_vector,
                    },
                    sender: tx,
                })
                .unwrap();
            receivers.push(rx);
        }
    }
    let mut data = vec![T::default(); matrix_len];
    for rx in receivers {
        let MsgOutput { idx, val } = rx.recv()?;
        data[idx] = val;
    }

    Ok(Matrix::new(data, a.row, b.col))
}

impl<T> Mul for Matrix<T>
where
    T: Mul<Output = T> + Add<Output = T> + Copy + Default + Send + 'static,
{
    type Output = Matrix<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).expect("matrix multiply error")
    }
}

impl<T> Display for Matrix<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for i in 0..self.row {
            for j in 0..self.col {
                write!(f, "{}", self.data[i * self.col + j])?;
                if j < self.col - 1 {
                    write!(f, " ")?;
                }
            }
            if i < self.row - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")
    }
}

impl<T> Debug for Matrix<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = a * b;
        assert_eq!(c.col, 2);
        assert_eq!(c.row, 2);
        assert_eq!(c.data, vec![22, 28, 49, 64]);
        assert_eq!(format!("{:?}", c), "Matrix(row=2, col=2, {22 28, 49 64})");

        Ok(())
    }

    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4], 2, 2);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = a * b;
        assert_eq!(c.data, vec![7, 10, 15, 22]);
        assert_eq!(format!("{}", c), "{7 10, 15 22}");
        Ok(())
    }

    #[test]
    fn test_a_can_not_multiply_b() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b);
        assert!(c.is_err());
    }

    #[test]
    #[should_panic]
    fn test_a_can_not_multiply_b_panic() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let _c = a * b;
    }
}
