/// Just a rectangular matrix, nothing fancy and probably not what you are
/// looking for. Check `src/transform.rs` for a real transformation matrix.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Matrix<T> {
    contents: Vec<T>,
    width: usize,
}

impl<T: Copy> Matrix<T> {
    /// Build a matrix from its parts.
    ///
    /// # Panics
    ///
    /// * If `width` is zero
    /// * If `width` doesn't divide the vector's length
    pub fn from_parts(contents: Vec<T>, width: usize) -> Matrix<T> {
        assert_ne!(width, 0);
        assert_eq!(contents.len() % width, 0);

        Matrix { contents, width }
    }

    pub fn row_count(&self) -> usize {
        self.contents.len() / self.width
    }

    pub fn col_count(&self) -> usize {
        self.width
    }

    pub fn rows(&self) -> Vec<&[T]> {
        self.contents.chunks_exact(self.width).collect()
    }

    pub fn cols(&self) -> Vec<Vec<T>> {
        let mut output = Vec::with_capacity(self.width);
        let col_num = self.width;
        let row_num = self.contents.len() / self.width;

        for i in 0..col_num {
            // add every column to the output
            let mut col = Vec::with_capacity(self.contents.len() / self.width);

            for j in 0..row_num {
                col.push(self.contents[j * col_num + i]);
            }

            output.push(col);
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_can_iterate_rows_and_cols() {
        let matrix = Matrix::from_parts(vec![1, 2, 3, 4, 5, 6], 3);

        let expected_rows = [[1, 2, 3], [4, 5, 6]];

        for (orig, new) in matrix.rows().iter().zip(expected_rows.iter()) {
            assert_eq!(orig, new);
        }

        let expected_cols = [[1, 4], [2, 5], [3, 6]];

        assert_eq!(matrix.cols().len(), 3);

        for (orig, new) in matrix.cols().into_iter().zip(expected_cols.iter()) {
            assert_eq!(orig, new);
        }
    }
}
