//! Response shapes for grid → member matching (docs/16).
//!
//! The members are inferred, never stored, so the response carries its own
//! confidence and the axes it reasoned from — a client showing "these are the
//! images this grid is made of" needs to be able to say how sure that is.

use crate::dto::image::ImageListItem;
use crate::http::warnings::Warning;

/// One axis of the grid.
#[derive(Debug, serde::Serialize)]
pub struct GridAxis {
    /// The label A1111 recorded, verbatim ("CFG Scale", "Prompt S/R", …).
    #[serde(rename = "type")]
    pub axis_type: String,
    pub values: Vec<String>,
    /// The image field this axis varies, or null when the type is one the
    /// matcher does not understand (members then rest on the other axes).
    pub column: Option<&'static str>,
}

/// The X/Y/Z axes. Absent axes are null — a 2D grid has no `z`.
#[derive(Debug, Default, serde::Serialize)]
pub struct GridAxes {
    pub x: Option<GridAxis>,
    pub y: Option<GridAxis>,
    pub z: Option<GridAxis>,
}

/// Zero-based position of a member within the montage.
#[derive(Debug, serde::Serialize)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

/// The axis value each position corresponds to, for labelling cells.
#[derive(Debug, serde::Serialize)]
pub struct GridAxisValues {
    pub x: Option<String>,
    pub y: Option<String>,
    pub z: Option<String>,
}

/// A member image plus where it sits on the grid. The image fields are inlined,
/// so a client can hand this straight to the same card component it uses for a
/// listing.
#[derive(Debug, serde::Serialize)]
pub struct GridMember {
    #[serde(flatten)]
    pub image: ImageListItem,
    pub position: GridPosition,
    pub axis_values: GridAxisValues,
}

/// GET /api/images/{id}/grid-members
#[derive(Debug, serde::Serialize)]
pub struct GridMembersResponse {
    /// The grid itself, so a client can render the page from one request.
    pub grid: ImageListItem,
    /// Null when the grid carries no usable axis metadata at all.
    pub axes: Option<GridAxes>,
    /// Ordered by axis position (z, then y, then x).
    pub members: Vec<GridMember>,
    /// Cells the axes imply, when every axis type is understood.
    pub expected_cells: Option<usize>,
    pub matched: usize,
    /// How much to trust the list:
    /// - `exact` — every implied cell was found;
    /// - `partial` — fewer (or more) than implied, e.g. cells were deleted;
    /// - `heuristic` — no axis type was understood, so only the invariant
    ///   parameters and the time window narrowed it;
    /// - `none` — no axis metadata, nothing was matched.
    pub confidence: &'static str,
    /// The window actually used, after clamping.
    pub window_hours: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Warning>,
}
