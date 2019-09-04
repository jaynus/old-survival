% Delaunay2D

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
