/// Just a rectangular matrix
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Matrix<T> {
    contents: Vec<T>,
    width: usize,
}

impl<T> Matrix<T> {
    /// Build a matrix from its parts.
    ///
    /// # Panics
    ///
    /// * If `width` is zero
    /// * If `width` doesn't divide the vector's length
    pub fn from_parts(contents: Vec<T>, width: usize) -> Matrix<T> {
        assert_ne!(width, 0);
        assert_eq!(contents.len() % width, 0);

        Matrix {
            contents, width,
        }
    }

    pub fn row_count(&self) -> usize {
        self.contents.len() / self.width
    }

    pub fn col_count(&self) -> usize {
        self.width
    }

    pub fn rows(&self) -> &[T] {
        todo!()
    }
}
