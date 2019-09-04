use crate::{DCEL, Point, voronoi, lloyd_relaxation, make_polygons, polygon_centroid};
use crate::dcel::NIL;
use rayon::prelude::*;

#[derive(Debug)]
pub struct VoronoiDiagram {
    /// DCEL is public allowing low-level access
    pub dcel: DCEL,
}
impl VoronoiDiagram {
    pub fn new(points: &[Point], boxsize: f64, lloyd_relaxation_passes: usize) -> Self {
        let mut p = points.to_vec();
        (0..lloyd_relaxation_passes).for_each(|_| {
            p = lloyd_relaxation(&p, boxsize);
        });

        Self {
            dcel: voronoi(points, boxsize),
        }
    }

    pub fn cells(&self) -> Vec<VoronoiCell> {
        let polygons = make_polygons(&self.dcel);

        let mut ret = polygons.iter().map(|points| {
            VoronoiCell {
                points: points.clone(),
                centroid: polygon_centroid(points),
            }
        })
        .collect::<Vec<_>>();
        ret.sort();
        ret
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VoronoiCell {
    pub points: Vec<Point>,
    pub centroid: Point,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voronoi;

    #[test]
    fn wrapper_readme_example() {
        let vor_pts = vec![Point::new(0.0, 1.0), Point::new(2.0, 3.0), Point::new(10.0, 12.0)];
        let vor_diagram = VoronoiDiagram::new(&vor_pts, 800., 2);
        // This returns 4 because we cant skip the outer face
        assert_eq!(vor_diagram.cells().count(), 4);
    }
}