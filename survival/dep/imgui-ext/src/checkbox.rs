//! ## Optional fields
//!
//! * `label`
//! * `catch`
//!
//! ## Example
//!
//! ```
//! use imgui_ext::ImGuiExt;
//!
//! #[derive(ImGuiExt)]
//! struct Checkboxes {
//!     // All parameters are optional.
//!     #[imgui(checkbox)]
//!     turbo: bool,
//!
//!     // Optionally, you can override the label:
//!     #[imgui(checkbox(label = "Checkbox!"))]
//!     check: bool,
//! }
//! ```
//!
//! ### Result
//!
//! ![][result]
//!
//! [result]: https://i.imgur.com/1hTR89V.png
use imgui::{ImStr, Ui};
use std::pin::Pin;

/// Structure generated by the annoration.
#[derive(Copy, Clone)]
pub struct CheckboxParams<'ui> {
    pub label: &'ui ImStr,
}

/// Trait for types that can be represented with a checkbox.
pub trait Checkbox {
    fn build(ui: &Ui, elem: &mut Self, params: CheckboxParams) -> bool;
}

impl<C: Checkbox> Checkbox for Option<C> {
    fn build(ui: &Ui, elem: &mut Self, params: CheckboxParams) -> bool {
        if let Some(ref mut elem) = elem {
            C::build(ui, elem, params)
        } else {
            false
        }
    }
}

impl Checkbox for bool {
    fn build(ui: &Ui, elem: &mut Self, params: CheckboxParams) -> bool {
        ui.checkbox(params.label, elem)
    }
}

impl<T: Checkbox> Checkbox for Box<T> {
    #[inline]
    fn build(ui: &Ui, elem: &mut Self, params: CheckboxParams) -> bool {
        T::build(ui, elem, params)
    }
}

impl<T: Checkbox + Unpin> Checkbox for Pin<Box<T>> {
    fn build(ui: &Ui, elem: &mut Self, params: CheckboxParams) -> bool {
        T::build(ui, elem.as_mut().get_mut(), params)
    }
}