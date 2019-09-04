#![deny(missing_debug_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unused_import_braces, unused_qualifications)]
#![allow(missing_docs, unstable_features)]
#![feature(crate_visibility_modifier)]

//! A Rust implementation of Fortune's Linesweep algorithm for computing Voronoi diagrams.

#[macro_use]
extern crate log;

#[cfg(feature = "serde_support")]
extern crate serde;

#[cfg(feature = "serde_support")]
#[macro_use]
extern crate serde_derive;

mod geometry;
mod point;
mod dcel;
mod beachline;
mod event;
mod voronoi;
mod lloyd;
mod wrapper;

pub use crate::voronoi::voronoi;
pub use crate::point::Point;
pub use crate::dcel::{DCEL, make_line_segments, make_polygons};
pub use crate::lloyd::{lloyd_relaxation, polygon_centroid};

pub use crate::wrapper::{VoronoiDiagram, VoronoiCell};