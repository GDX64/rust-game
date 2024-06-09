pub struct SparseMatrix<T> {
    data: NegativeVector<NegativeVector<T>>,
}

impl<T> SparseMatrix<T> {
    pub fn new() -> Self {
        Self {
            data: NegativeVector::new(),
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&T> {
        self.data.get(x).and_then(|x| x.get(y))
    }

    pub fn set(&mut self, x: i32, y: i32, value: T) {
        if let Some(row) = self.data.get_mut(x) {
            row.set(y, value);
        } else {
            let mut row = NegativeVector::new();
            row.set(y, value);
            self.data.set(x, row);
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

struct NegativeVector<T> {
    positive: Vec<Option<T>>,
    negative: Vec<Option<T>>,
}

impl<T> NegativeVector<T> {
    pub fn new() -> Self {
        Self {
            positive: Vec::new(),
            negative: Vec::new(),
        }
    }

    pub fn get_mut(&mut self, index: i32) -> Option<&mut T> {
        if index >= 0 {
            let index = index as usize;
            self.positive.get_mut(index).and_then(|x| x.as_mut())
        } else {
            let index = -index as usize;
            self.negative.get_mut(index).and_then(|x| x.as_mut())
        }
    }

    pub fn get(&self, index: i32) -> Option<&T> {
        if index >= 0 {
            let index = index as usize;
            self.positive.get(index).and_then(|x| x.as_ref())
        } else {
            let index = -index as usize;
            self.negative.get(index).and_then(|x| x.as_ref())
        }
    }

    pub fn set(&mut self, index: i32, value: T) {
        if index >= 0 {
            let index = index as usize;
            while let None = self.positive.get(index) {
                self.positive.push(None);
            }
            self.positive[index] = Some(value);
        } else {
            let index = -index as usize;
            while let None = self.negative.get(index) {
                self.negative.push(None);
            }
            self.negative[index] = Some(value);
        }
    }

    pub fn clear(&mut self) {
        self.positive.clear();
        self.negative.clear();
    }
}

#[cfg(test)]
mod test {
    use crate::sparse_matrix::NegativeVector;

    #[test]
    fn test_insert_array() {
        let mut v = NegativeVector::new();
        v.set(1, 1);
        v.set(-1, 2);
        assert_eq!(v.get(1), Some(&1));
        assert_eq!(v.get(-1), Some(&2));
    }

    #[test]
    fn test_insert_array_sparse() {
        let mut v = NegativeVector::new();
        v.set(10, 1);
        v.set(-10, 2);
        assert_eq!(v.get(1), None);
        assert_eq!(v.get(-1), None);

        assert_eq!(v.get(10), Some(&1));
        assert_eq!(v.get(-10), Some(&2));
    }

    #[test]
    fn test_sparse_matrix() {
        let mut matrix = super::SparseMatrix::<i32>::new();
        assert_eq!(matrix.get(0, 0), None);
        matrix.set(10, 15, 10);

        assert_eq!(matrix.get(10, 15), Some(&10));
        assert_eq!(matrix.get(1, 9), None);
    }
}
