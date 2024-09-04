use crate::{game_map::V2D, wasm_game::ShipKey};

#[derive(Clone)]
pub struct HashEntity {
    pub position: V2D,
    pub entity: HashEntityKind,
}

#[derive(Clone)]
pub enum HashEntityKind {
    Boat(ShipKey),
    Lighthouse(u64),
}

#[derive(Clone)]
pub struct HashGrid {
    tile_size: f64,
    entities: Vec<Option<Vec<HashEntity>>>,
    dim: f64,
    tiles_dim: usize,
}

impl HashGrid {
    pub fn new(dim: f64, tile_size: f64) -> Self {
        let tiles_dim = (dim / tile_size).ceil() as usize;
        let dim_square = tiles_dim * tiles_dim;
        let entities = vec![None; dim_square];
        Self {
            tile_size,
            entities,
            dim,
            tiles_dim,
        }
    }

    fn tile_unit(&self, val: f64) -> usize {
        ((val + self.dim / 2.0) / self.tile_size) as usize
    }

    fn bucket_number_of(&self, v: &V2D) -> usize {
        let bucket_x = self.tile_unit(v.x);
        let bucket_y = self.tile_unit(v.y);
        let i: usize = bucket_x + bucket_y * self.tiles_dim;
        return i;
    }

    pub fn insert(&mut self, entity: HashEntity) -> Option<()> {
        let i = self.bucket_number_of(&entity.position);
        let bucket = self.entities.get_mut(i)?;
        let bucket = if let Some(bucket) = bucket {
            bucket
        } else {
            *bucket = Some(vec![]);
            let bucket = bucket.as_mut()?;
            bucket
        };
        bucket.push(entity);
        Some(())
    }

    pub fn query_near<'a>(&'a self, v: &V2D) -> impl Iterator<Item = &'a HashEntity> {
        let entities = self
            .near_buckets(v)
            .into_iter()
            .flat_map(|bucket| self.entities.get(bucket as usize))
            .flat_map(|bucket| bucket)
            .flat_map(|bucket| bucket);
        return entities;
    }

    fn near_buckets(&self, v: &V2D) -> [i32; 9] {
        let mut candidates = [0i32; 9];
        let x = self.tile_unit(v.x) as i32;
        let y = self.tile_unit(v.y) as i32;
        let tiles_dim = self.tiles_dim as i32;
        candidates[0] = x + y * tiles_dim;
        candidates[1] = x + 1 + y * tiles_dim;
        candidates[2] = x + 1 + (y + 1) * tiles_dim;
        candidates[3] = x + (y + 1) * tiles_dim;
        candidates[4] = x - 1 + (y + 1) * tiles_dim;
        candidates[5] = x - 1 + (y) * tiles_dim;
        candidates[6] = x - 1 + (y + 1) * tiles_dim;
        candidates[7] = x + (y + 1) * tiles_dim;
        candidates[8] = x + 1 + (y + 1) * tiles_dim;
        return candidates;
    }
}

#[cfg(test)]
mod test {
    use super::{HashEntity, HashEntityKind, HashGrid};

    #[test]
    fn test_grid() {
        let mut grid = HashGrid::new(1000.0, 100.0);
        let e1 = HashEntity {
            entity: HashEntityKind::Lighthouse(1),
            position: (10.0, 10.0).into(),
        };
        grid.insert(e1);

        let e2 = HashEntity {
            entity: HashEntityKind::Lighthouse(1),
            position: (120.0, 10.0).into(),
        };

        grid.insert(e2);

        let iter = grid.query_near(&((0.0, 0.0).into()));
        let count = iter.count();
        assert_eq!(count, 2);

        let e3 = HashEntity {
            entity: HashEntityKind::Lighthouse(1),
            position: (-90.0, 10.0).into(),
        };
        let e4 = HashEntity {
            entity: HashEntityKind::Lighthouse(1),
            position: (-120.0, 10.0).into(),
        };
        grid.insert(e3);
        grid.insert(e4);

        let iter = grid.query_near(&((0.0, 0.0).into()));
        let count = iter.count();
        assert_eq!(count, 3);
    }
}