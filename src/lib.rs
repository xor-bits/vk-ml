use std::{marker::PhantomData, ops::Add};

use self::back::Handle;

//

mod back;

//

pub struct Matrix<T, const ROWS: usize, const COLS: usize> {
    _p: PhantomData<T>,
}

impl<T, const ROWS: usize, const COLS: usize> Matrix<T, ROWS, COLS> {
    pub const fn zeros() -> Self {
        Self { _p: PhantomData }
    }
}

impl<T, const ROWS: usize, const COMMON: usize, const COLS: usize> Add<Matrix<T, COMMON, COLS>>
    for Matrix<T, ROWS, COMMON>
{
    type Output = Matrix<T, ROWS, COLS>;

    fn add(self, rhs: Matrix<T, COMMON, COLS>) -> Self::Output {
        Self::Output::zeros()
    }
}

//

pub struct Tensor<T, const LVL: usize = 0> {
    shape: Shape<LVL>,
    handle: Handle,
    _p: PhantomData<T>,
}

impl<T> Tensor<T, 0> {
    pub const fn empty<const LVL: usize>(dimensions: [usize; LVL]) -> Tensor<T, LVL> {
        Tensor {
            shape: Shape::new(dimensions),
            handle: Handle,
            _p: PhantomData,
        }
    }
}

impl<T, const LVL: usize> Tensor<T, LVL> {
    pub fn dbg(&self) {
        let be = back::backend();
        dbg!(&be);
    }
}

//

pub struct Shape<const LVL: usize> {
    pub dimensions: [usize; LVL],
}

impl<const LVL: usize> Shape<LVL> {
    pub const fn new(dimensions: [usize; LVL]) -> Self {
        Self { dimensions }
    }
}

impl<const LVL: usize> From<[usize; LVL]> for Shape<LVL> {
    fn from(value: [usize; LVL]) -> Self {
        Self::new(value)
    }
}
