use num_traits::Float;

/// The datastructure holding the information for each point of the scattered dataset
/// to interpolate values from.
#[derive(Clone, Copy, Debug)]
pub struct PtValue<T> {
    /// The x coordinate.
    pub x: T,
    /// The y coordinate.
    pub y: T,
    /// The value associated to these coordinates.
    pub value: T,
}

impl<T> PtValue<T>
    where T: Float
{
    /// Create a new PtValue given a x, a y and a value.
    pub fn new(x: T, y: T, value: T) -> Self {
        PtValue {
            x: x,
            y: y,
            value: value,
        }
    }
    pub fn get_coordinates(&self) -> (T, T) {
        (self.x, self.y)
    }
    pub fn get_value(&self) -> T {
        self.value
    }
    pub fn set_value(&mut self, value: T) {
        self.value = value;
    }
    pub fn get_triplet(&self) -> (T, T, T) {
        (self.x, self.y, self.value)
    }
}

impl<T> From<nalgebra::Vector3<T>> for PtValue<T>
    where T: 'static + Float + std::fmt::Debug
{
    fn from(v: nalgebra::Vector3<T>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            value: v.z,
        }
    }
}

