/*!
This crate provides a library for computing a [Delaunay triangulation][1] from a set of points
using the [Bowyer–Watson algorithm][2].

While performance shouldn't be terrible, there is definite room for improvement.

# Usage
```rust
extern crate delaunay2d;
```

# Example: Delaunay triangulation
```rust
use delaunay2d::{Delaunay2D, Triangle};
let mut dt = Delaunay2D::new((0., 0.), 9999.);

dt.add_point((13., 12.));
dt.add_point((18., 19.));
dt.add_point((21., 5.));
dt.add_point((37., -3.));

let mut triangles = dt.export_triangles();
triangles.sort_by_key(|t| (t.0, t.1, t.2));
assert_eq!(vec![Triangle(2,1,0), Triangle(3, 1, 2)], triangles);
```
# Example: Voronoi regions
```rust
use delaunay2d::{Delaunay2D};
let mut dt = Delaunay2D::new((0., 0.), 9999.);

dt.add_point((13., 12.));
dt.add_point((18., 19.));
dt.add_point((21., 5.));
dt.add_point((37., -3.));

let (points, mut regions) = dt.export_voronoi_regions();
regions.sort();
assert_eq!(10, points.len());
assert_eq!(4, regions.len());
```

[1]: https://en.wikipedia.org/wiki/Delaunay_triangulation
[2]: https://en.wikipedia.org/wiki/Bowyer%E2%80%93Watson_algorithm
*/

use std::ops::Add;
use std::ops::Sub;
use std::collections::HashMap;
use std::collections::HashSet;

#[inline]
fn next_idx(n: usize, m: usize) -> usize {
    if n < (m - 1) { n + 1 } else { 0 }
}

#[inline]
fn prev_idx(n: usize, m: usize) -> usize {
    if n == 0 { m - 1 } else { n - 1 }
}

/// Represents an (X, Y) coordinate
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Point {
        Point { x: x, y: y }
    }
    fn mag(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

/// The triangles opposite to each vertex, if any.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TNeighbours(pub Option<Triangle>, pub Option<Triangle>, pub Option<Triangle>);

impl TNeighbours {
    fn get(&self, n: usize) -> Option<Triangle> {
        match n {
            0 => self.0,
            1 => self.1,
            2 => self.2,
            _ => None,
        }
    }
    fn get_ccw_op(&self, t: Triangle) -> usize {
        match *self {
            TNeighbours(Some(x), _, _) if x == t => 0,
            TNeighbours(_, Some(x), _) if x == t => 1,
            TNeighbours(_, _, Some(x)) if x == t => 2,
            _ => panic!("At the Disco"),
        }
    }

    fn update_with_neighbour(self, e0: usize, e1: usize, t: Triangle) -> TNeighbours {
        match self {
            TNeighbours(Some(x), b, c) if x.has_edges(e0, e1) => TNeighbours(Some(t), b, c),
            TNeighbours(a, Some(x), c) if x.has_edges(e0, e1) => TNeighbours(a, Some(t), c),
            TNeighbours(a, b, Some(x)) if x.has_edges(e0, e1) => TNeighbours(a, b, Some(t)),
            _ => self,
        }
    }
    fn remove_bounding_triangles(&self) -> TNeighbours {
        TNeighbours(self.0.and_then(|t| if t.is_bounding_triangle() {
                        None
                    } else {
                        Some(t)
                    }),
                    self.1.and_then(|t| if t.is_bounding_triangle() {
                        None
                    } else {
                        Some(t)
                    }),
                    self.2.and_then(|t| if t.is_bounding_triangle() {
                        None
                    } else {
                        Some(t)
                    }))
    }

    fn munge_indices(&self) -> TNeighbours {
        TNeighbours(self.0.map(|t| t.munge_indices()),
                    self.1.map(|t| t.munge_indices()),
                    self.2.map(|t| t.munge_indices()))
    }
}

#[derive(Debug)]
pub struct Delaunay2D {
    coords: Vec<Point>,
    triangles: HashMap<Triangle, TNeighbours>,
    circles: HashMap<Triangle, (Point, f64)>,
}

/// A triangle, represented as indices into a list of points
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Triangle(pub usize, pub usize, pub usize);

impl Triangle {
    fn get(&self, n: usize) -> usize {
        match n {
            0 => self.0,
            1 => self.1,
            2 => self.2,
            _ => panic!("Triangles only have three sides"),
        }
    }

    fn has_edges(&self, e0: usize, e1: usize) -> bool {
        (self.0 == e0 || self.1 == e0 || self.2 == e0) &&
        (self.0 == e1 || self.1 == e1 || self.2 == e1)
    }

    fn is_bounding_triangle(&self) -> bool {
        self.0 <= 3 || self.1 <= 3 || self.2 <= 3
    }

    fn munge_indices(&self) -> Self {
        Triangle(self.0 - 4, self.1 - 4, self.2 - 4)
    }
    fn demunge_indices(&self) -> Self {
        Triangle(self.0 + 4, self.1 + 4, self.2 + 4)
    }
}

impl Delaunay2D {
    /// Builds a new Delaunay triangulator.
    /// All points added to the triangulator must fall within the bounding box
    /// centered at `center` and extending in `radius` in each direction
    pub fn new(center: (f64, f64), radius: f64) -> Delaunay2D {
        let center = Point::new(center.0, center.1);
        let coords = vec![Point {
                              x: center.x - radius,
                              y: center.y - radius,
                          },
                          Point {
                              x: center.x + radius,
                              y: center.y - radius,
                          },
                          Point {
                              x: center.x + radius,
                              y: center.y + radius,
                          },
                          Point {
                              x: center.x - radius,
                              y: center.y + radius,
                          }];

        let mut triangles = HashMap::new();
        let mut circles = HashMap::new();
        let t1 = Triangle(0, 1, 3);
        let t2 = Triangle(2, 3, 1);
        triangles.insert(t1, TNeighbours(Some(t2), None, None));
        triangles.insert(t2, TNeighbours(Some(t1), None, None));
        circles.insert(t1, t1.circumcenter(&coords));
        circles.insert(t2, t2.circumcenter(&coords));
        Delaunay2D {
            coords: coords,
            triangles: triangles,
            circles: circles,
        }
    }

    fn in_circle_fast(&self, tri: Triangle, p: Point) -> bool {
        let (center, radius) = self.circles[&tri];
        (center - p).mag() <= radius
    }

    // fn in_circle_robust(&self, tri: Triangle, p: Point) -> bool {
    // 	let (a, b, c) = (self.coords[tri.0] - p, self.coords[tri.1] - p, self.coords[tri.2] - p);
    // 	let a_mag = a.mag();
    // 	let b_mag = b.mag();
    // 	let c_mag = c.mag();
    // 	let det = a.x * (b.y * c_mag - b_mag * c.y)
    // 	        + a.y * (b_mag * c.x - c_mag * b.x)
    // 	        + a_mag * (b.x * c.y - c.x * b.y);

    // 	det > 0f64
    // }

    /// Adds a point to the triangulation.
    #[allow(while_true)]
    pub fn add_point(&mut self, p: (f64, f64)) {
        let p = Point::new(p.0, p.1);
        let idx = self.coords.len();
        self.coords.push(p);

        let bad_triangles: HashSet<_> =
            self.triangles.keys().cloned().filter(|&t| self.in_circle_fast(t, p)).collect();

        let mut boundary: Vec<(usize, usize, Option<Triangle>)> = vec![];
        let mut t: Triangle = *bad_triangles.iter().next().unwrap();
        let mut edge = 0;

        while true {
            // Check if edge of triangle T is on the boundary...
            // if opposite triangle of this edge is external to the list
            let tri_op = self.triangles[&t].get(edge);
            if tri_op.is_none() || !bad_triangles.contains(&tri_op.unwrap()) {

                // Insert edge and external triangle into boundary list
                boundary.push((t.get(next_idx(edge, 3)), t.get(prev_idx(edge, 3)), tri_op));
                // Move to next CCW edge in this triangle
                edge = next_idx(edge, 3);
                if boundary[0].0 == boundary[boundary.len() - 1].1 {
                    break;
                }
            } else if let Some(tri_op) = tri_op {
                // Move to next CCW edge in opposite triangle
                let ccw_op = self.triangles[&tri_op].get_ccw_op(t);
                //edge = (self.triangles[tri_op].index(T) + 1) % 3
                edge = next_idx(ccw_op, 3);
                t = tri_op;
            }
        }
        for t in bad_triangles {
            self.triangles.remove(&t);
            self.circles.remove(&t);
        }
        // Retriangle the hole left by bad_triangles
        let mut new_triangles: Vec<Triangle> = vec![];
        for (e0, e1, tri_op) in boundary {
            // Create a new triangle using point p and edge extremes
            let t = Triangle(idx, e0, e1);
            // Store circumcenter and circumradius of the triangle
            self.circles.insert(t, t.circumcenter(&self.coords));
            // Set opposite triangle of the edge as neighbour of T
            self.triangles.insert(t, TNeighbours(tri_op, None, None));

            // Try to set T as neighbour of the opposite triangle
            // search the neighbour of tri_op that use edge (e1, e0)
            if let Some(tri_op) = tri_op {
                let updated_tstruct = self.triangles[&tri_op].update_with_neighbour(e0, e1, t);
                self.triangles.insert(tri_op, updated_tstruct);
            }

            // Add triangle to a temporal list
            new_triangles.push(t);

        }

        // Link the new triangles each another
        let n = new_triangles.len();
        for (i, t) in new_triangles.iter().enumerate() {
            let tstruct = self.triangles[t];
            let first_triangle = new_triangles[next_idx(i, n)];
            let second_triangle = new_triangles[prev_idx(i, n)];
            let new_tstruct = TNeighbours(tstruct.0, Some(first_triangle), Some(second_triangle));
            self.triangles.insert(*t, new_tstruct);
        }
    }

    /// Returns the triangles generated by the triangulation.
    /// Each triangle is a counter-clockwise triple of coordinate indices
    pub fn export_triangles(&self) -> Vec<Triangle> {
        let mut ret = self.triangles
            .keys()
            .filter(|t| t.0 > 3 && t.1 > 3 && t.2 > 3)
            .cloned()
            .map(|t| t.munge_indices())
            .collect::<Vec<_>>();
        ret.sort();
        ret
    }

    /// Returns the neighbours of a given triangle.
    /// The first neighbour is adjacent to the edge *opposite* the first vertex, etc.
    pub fn get_adjacent(&self, t: &Triangle) -> Option<TNeighbours> {
        // Because we munge outgoing triangles, we need to demunge them to look them up
        let demunged = t.demunge_indices();
        self.triangles.get(&demunged).map(|t| t.remove_bounding_triangles().munge_indices())
    }

    /// Returns the list of points added to the triangulation.
    pub fn export_points(&self) -> Vec<(f64, f64)> {
        self.coords.iter().skip(4).map(|p| (p.x, p.y)).collect()
    }

    /// Returns the vertices of the Voronoi regions, and the indices of vertices forming
    /// each region.
    pub fn export_voronoi_regions(&self) -> (Vec<(f64, f64)>, Vec<Vec<usize>>) {
        let mut use_vertex =
            (0..self.coords.len()).map(|_| -> Vec<Triangle> { vec![] }).collect::<Vec<_>>();
        let mut index: HashMap<Triangle, usize> = HashMap::new();
        let mut vor_coors = vec![];
        for (tidx, t) in self.triangles.keys().enumerate() {
            let Triangle(a, b, c) = *t;
            let circle_center = self.circles[t].0;

            vor_coors.push((circle_center.x, circle_center.y));
            // Insert triangle, rotating it so the key is the "last" vertex
            use_vertex[a].push(Triangle(b, c, a));
            use_vertex[b].push(Triangle(c, a, b));
            use_vertex[c].push(Triangle(a, b, c));
            // Set tidx as the index to use with this triangles
            index.insert(Triangle(b, c, a), tidx);
            index.insert(Triangle(c, a, b), tidx);
            index.insert(Triangle(a, b, c), tidx);
        }
        // init regions per coordinate dictionary
        let mut regions = vec![];
        // Sort each region in a coherent order, and substitude each triangle
        // by its index

        for vertex in use_vertex[4..].iter() {
            let mut v = vertex[0].0;
            let mut r = vec![];
            for _ in 0..vertex.len() {
                // Search the triangle beginning with vertex v
                let t = vertex.iter().find(|&t| t.0 == v).unwrap();
                r.push(index[&t]); // Add the index of this triangle to region
                v = t.1; // Choose the next vertex to search
            }
            regions.push(r); // Store region.
        }
        (vor_coors, regions)
    }
}

impl Triangle {
    fn circumcenter(&self, coords: &[Point]) -> (Point, f64) {

        let (a, b, c) = (coords[self.0], coords[self.1], coords[self.2]);
        /* Use coordinates relative to point `a' of the triangle. */
        let ba = b - a;
        let ca = c - a;
        // Squares of lengths of the edges incident to `a`
        let ba_length = ba.mag();
        let ca_length = ca.mag();

        // Living dangerously
        let denominator = 0.5 / (ba.x * ca.y - ba.y * ca.x);

        let xcirca = (ca.y * ba_length - ba.y * ca_length) * denominator;
        let ycirca = (ba.x * ca_length - ca.x * ba_length) * denominator;

        let a_relative_circumcenter = Point {
            x: xcirca,
            y: ycirca,
        };

        let r_squared = a_relative_circumcenter.mag();

        (a + a_relative_circumcenter, r_squared)
    }
}


#[cfg(test)]
mod tests {
    use Delaunay2D;
    use Triangle;
    use TNeighbours;
    #[test]
    fn it_works() {
        let mut delaunay = Delaunay2D::new((0., 0.), 9999.);
        delaunay.add_point((13., 12.));
        for &(t, tstruct) in [(Triangle(4, 0, 1),
                               TNeighbours(None,
                                           Some(Triangle(4, 1, 2)),
                                           Some(Triangle(4, 3, 0)))),
                              (Triangle(4, 1, 2),
                               TNeighbours(None,
                                           Some(Triangle(4, 2, 3)),
                                           Some(Triangle(4, 0, 1)))),
                              (Triangle(4, 3, 0),
                               TNeighbours(None,
                                           Some(Triangle(4, 0, 1)),
                                           Some(Triangle(4, 2, 3)))),
                              (Triangle(4, 2, 3),
                               TNeighbours(None,
                                           Some(Triangle(4, 3, 0)),
                                           Some(Triangle(4, 1, 2))))]
            .into_iter() {
            assert!(delaunay.triangles.contains_key(&t));
            let ts = delaunay.triangles[&t];
            assert_eq!(tstruct, ts);
        }

        delaunay.add_point((18., 19.));
        for &(t, tstruct) in [(Triangle(4, 0, 1),
                               TNeighbours(None,
                                           Some(Triangle(5, 4, 1)),
                                           Some(Triangle(4, 3, 0)))),
                              (Triangle(5, 2, 3),
                               TNeighbours(None,
                                           Some(Triangle(5, 3, 4)),
                                           Some(Triangle(5, 1, 2)))),
                              (Triangle(5, 1, 2),
                               TNeighbours(None,
                                           Some(Triangle(5, 2, 3)),
                                           Some(Triangle(5, 4, 1)))),
                              (Triangle(5, 3, 4),
                               TNeighbours(Some(Triangle(4, 3, 0)),
                                           Some(Triangle(5, 4, 1)),
                                           Some(Triangle(5, 2, 3)))),
                              (Triangle(5, 4, 1),
                               TNeighbours(Some(Triangle(4, 0, 1)),
                                           Some(Triangle(5, 1, 2)),
                                           Some(Triangle(5, 3, 4)))),
                              (Triangle(4, 3, 0),
                               TNeighbours(None,
                                           Some(Triangle(4, 0, 1)),
                                           Some(Triangle(5, 3, 4))))]
            .into_iter() {
            assert!(delaunay.triangles.contains_key(&t));
            let ts = delaunay.triangles[&t];
            assert_eq!(tstruct, ts);
        }

        delaunay.add_point((21., 5.));
        for &(t, tstruct) in [(Triangle(6, 2, 5),
                               TNeighbours(Some(Triangle(5, 2, 3)),
                                           Some(Triangle(6, 5, 4)),
                                           Some(Triangle(6, 1, 2)))),
                              (Triangle(5, 2, 3),
                               TNeighbours(None,
                                           Some(Triangle(5, 3, 4)),
                                           Some(Triangle(6, 2, 5)))),
                              (Triangle(5, 3, 4),
                               TNeighbours(Some(Triangle(4, 3, 0)),
                                           Some(Triangle(6, 5, 4)),
                                           Some(Triangle(5, 2, 3)))),
                              (Triangle(6, 5, 4),
                               TNeighbours(Some(Triangle(5, 3, 4)),
                                           Some(Triangle(6, 4, 0)),
                                           Some(Triangle(6, 2, 5)))),
                              (Triangle(6, 0, 1),
                               TNeighbours(None,
                                           Some(Triangle(6, 1, 2)),
                                           Some(Triangle(6, 4, 0)))),
                              (Triangle(4, 3, 0),
                               TNeighbours(None,
                                           Some(Triangle(6, 4, 0)),
                                           Some(Triangle(5, 3, 4)))),
                              (Triangle(6, 1, 2),
                               TNeighbours(None,
                                           Some(Triangle(6, 2, 5)),
                                           Some(Triangle(6, 0, 1)))),
                              (Triangle(6, 4, 0),
                               TNeighbours(Some(Triangle(4, 3, 0)),
                                           Some(Triangle(6, 0, 1)),
                                           Some(Triangle(6, 5, 4))))]
            .into_iter() {
            assert!(delaunay.triangles.contains_key(&t));
            let ts = delaunay.triangles[&t];
            assert_eq!(tstruct, ts);
        }


        delaunay.add_point((37., -3.));

        for &(t, tstruct) in [(Triangle(5, 2, 3),
                               TNeighbours(None,
                                           Some(Triangle(5, 3, 4)),
                                           Some(Triangle(7, 2, 5)))),
                              (Triangle(7, 1, 2),
                               TNeighbours(None,
                                           Some(Triangle(7, 2, 5)),
                                           Some(Triangle(7, 0, 1)))),
                              (Triangle(6, 4, 0),
                               TNeighbours(Some(Triangle(4, 3, 0)),
                                           Some(Triangle(7, 6, 0)),
                                           Some(Triangle(6, 5, 4)))),
                              (Triangle(7, 0, 1),
                               TNeighbours(None,
                                           Some(Triangle(7, 1, 2)),
                                           Some(Triangle(7, 6, 0)))),
                              (Triangle(7, 6, 0),
                               TNeighbours(Some(Triangle(6, 4, 0)),
                                           Some(Triangle(7, 0, 1)),
                                           Some(Triangle(7, 5, 6)))),
                              (Triangle(5, 3, 4),
                               TNeighbours(Some(Triangle(4, 3, 0)),
                                           Some(Triangle(6, 5, 4)),
                                           Some(Triangle(5, 2, 3)))),
                              (Triangle(6, 5, 4),
                               TNeighbours(Some(Triangle(5, 3, 4)),
                                           Some(Triangle(6, 4, 0)),
                                           Some(Triangle(7, 5, 6)))),
                              (Triangle(7, 5, 6),
                               TNeighbours(Some(Triangle(6, 5, 4)),
                                           Some(Triangle(7, 6, 0)),
                                           Some(Triangle(7, 2, 5)))),
                              (Triangle(4, 3, 0),
                               TNeighbours(None,
                                           Some(Triangle(6, 4, 0)),
                                           Some(Triangle(5, 3, 4)))),
                              (Triangle(7, 2, 5),
                               TNeighbours(Some(Triangle(5, 2, 3)),
                                           Some(Triangle(7, 5, 6)),
                                           Some(Triangle(7, 1, 2))))]
            .into_iter() {
            assert!(delaunay.triangles.contains_key(&t));
            let ts = delaunay.triangles[&t];
            assert_eq!(tstruct, ts);
        }
        assert_eq!(10, delaunay.triangles.len());

    }

    #[test]
    fn adjacent_triangles() {
        let mut delaunay = Delaunay2D::new((0., 0.), 100.);
        delaunay.add_point((1., 1.));
        delaunay.add_point((3., 1.));
        delaunay.add_point((1., 3.));
        let triangles = delaunay.export_triangles();
        assert_eq!(1, triangles.len());
        let t = triangles[0];
        assert_eq!(Some(TNeighbours(None, None, None)),
                   delaunay.get_adjacent(&t));
        assert_eq!(None, delaunay.get_adjacent(&Triangle(1, 2, 4)));

        delaunay.add_point((3., 3.));
        let mut triangles = delaunay.export_triangles();
        assert_eq!(2, triangles.len());
        triangles.sort_by_key(|t| (t.0, t.1, t.2));
        let t = triangles[0];
        assert_eq!(Triangle(3, 0, 1), t);
        assert_eq!(Some(TNeighbours(None, None, Some(Triangle(3, 2, 0)))),
                   delaunay.get_adjacent(&t));

        assert_eq!(None, delaunay.get_adjacent(&Triangle(1, 2, 4)));

    }
}
