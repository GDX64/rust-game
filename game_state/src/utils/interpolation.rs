use cgmath::Point2;

#[derive(Clone)]
pub struct LinearInterpolation {
    points: Vec<Point2<f64>>,
}

impl LinearInterpolation {
    pub fn new(points: Vec<Point2<f64>>) -> Self {
        Self { points }
    }

    pub fn interpolate(&self, x: f64) -> Option<f64> {
        let first = self.points.first()?;
        let smaller_than_first = first.x >= x;
        if smaller_than_first {
            return first.y.into();
        }
        let last = self.points.last()?;
        let greater_than_last = last.x <= x;
        if greater_than_last {
            return last.y.into();
        }
        let index = self.points.iter().position(|p| p.x >= x)?;
        let first_point = self.points.get(index - 1)?;
        let next_point = self.points.get(index)?;
        let alpha = (next_point.y - first_point.y) / (next_point.x - first_point.x);
        let beta = first_point.y - alpha * first_point.x;
        Some(alpha * x + beta)
    }
}

#[cfg(test)]
mod test {
    use super::LinearInterpolation;
    use cgmath::Point2;

    #[test]
    fn test_basic() {
        let interpolation =
            LinearInterpolation::new(vec![Point2::new(0.0, 0.0), Point2::new(1.0, 1.0)]);
        assert_eq!(interpolation.interpolate(0.5), Some(0.5));
    }
    #[test]
    fn test_3_points() {
        let interpolation = LinearInterpolation::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(2.0, 5.0),
        ]);
        assert_eq!(interpolation.interpolate(0.5), Some(0.5));
        assert_eq!(interpolation.interpolate(1.5), Some(3.0));
    }
}
