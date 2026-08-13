//! Matching an XYZ grid to the images it was assembled from (docs/16).
//!
//! Nothing links a grid to its cells: A1111 writes the grid as a plain image
//! whose parameters describe the *axes* (`X Type` / `X Values` / …) and the base
//! settings, and the cells carry no reference back. Membership is therefore
//! inferred from three things at once:
//!
//!   1. every generation parameter the grid did **not** vary must be identical,
//!   2. the parameters it did vary must land on one of the axis values, and
//!   3. the image must have been created inside the window before the grid was
//!      saved (cells are generated first, the montage is written last).
//!
//! The result is an estimate, so the API reports a confidence alongside it.

use chrono::Duration;
use rust_decimal::prelude::ToPrimitive;
use serde_json::Value;
use sqlx::{PgPool, Postgres, QueryBuilder};

use super::model::ImageRow;

/// Most candidate members one grid may pull back. A grid of a few hundred cells
/// is already unusual; this only stops a pathological window from scanning the
/// whole library into memory.
pub const MAX_MEMBERS: i64 = 2000;

/// Slack on the "after" side of the window. The montage is written once every
/// cell exists, but file timestamps (and clocks) wobble.
const FORWARD_MARGIN_MINUTES: i64 = 10;

/// The `images` column an A1111 XYZ axis varies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisColumn {
    Seed,
    Steps,
    CfgScale,
    SamplerName,
    ModelName,
}

impl AxisColumn {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Seed => "seed",
            Self::Steps => "steps",
            Self::CfgScale => "cfg_scale",
            Self::SamplerName => "sampler_name",
            Self::ModelName => "model_name",
        }
    }

    /// Whether the SQL query may filter on this axis' values.
    ///
    /// `model_name` may not: A1111 writes the checkpoint axis the way its
    /// dropdown shows it (`model.safetensors [a1b2c3d4]`, sometimes with a
    /// subfolder), while `images.model_name` holds whatever the metadata
    /// carried. An equality test would quietly match nothing, so the column is
    /// only dropped from the invariants — and reconciled in Rust, where
    /// `model_key` can normalize both sides.
    fn filterable(self) -> bool {
        !matches!(self, Self::ModelName)
    }
}

/// Map an A1111 "X/Y/Z Type" label to the column it varies. Unknown labels
/// (`Prompt S/R`, `VAE`, …) yield `None`: they still count as an axis, they just
/// cannot narrow the query.
pub fn axis_column(axis_type: &str) -> Option<AxisColumn> {
    match axis_type.trim().to_ascii_lowercase().as_str() {
        "seed" => Some(AxisColumn::Seed),
        "steps" => Some(AxisColumn::Steps),
        "cfg scale" => Some(AxisColumn::CfgScale),
        "sampler" | "sampler name" => Some(AxisColumn::SamplerName),
        "checkpoint name" | "model" => Some(AxisColumn::ModelName),
        _ => None,
    }
}

/// Split an axis value list. A1111 writes them comma-separated, quoting entries
/// that contain spaces.
pub fn parse_axis_values(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().trim_matches('"').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// One axis of a grid, as recorded in `model_params`.
#[derive(Debug, Clone)]
pub struct Axis {
    /// "x", "y" or "z" — which slot of the montage this axis drives.
    pub name: &'static str,
    /// The label A1111 used, kept verbatim for display.
    pub axis_type: String,
    pub values: Vec<String>,
    /// The column `axis_type` maps onto, if the matcher understands it.
    pub column: Option<AxisColumn>,
}

/// Read the X/Y/Z axes out of an image's `model_params`. An axis missing either
/// its type or its values is skipped — a half-recorded axis cannot be matched
/// on and would only inflate the expected cell count.
pub fn axes_of(model_params: &Value) -> Vec<Axis> {
    let mut axes = Vec::new();
    for name in ["x", "y", "z"] {
        let axis_type = model_params
            .get(format!("xyz_{name}_type"))
            .and_then(Value::as_str);
        let raw_values = model_params
            .get(format!("xyz_{name}_values"))
            .and_then(Value::as_str);
        let (Some(axis_type), Some(raw_values)) = (axis_type, raw_values) else {
            continue;
        };
        let values = parse_axis_values(raw_values);
        if values.is_empty() {
            continue;
        }
        axes.push(Axis {
            name,
            column: axis_column(axis_type),
            axis_type: axis_type.to_string(),
            values,
        });
    }
    axes
}

/// Whether an image is itself a grid (`model_params.is_xyz_grid`). The parser
/// writes a JSON boolean; the filename fallback in the worker writes the same,
/// but a string is accepted so hand-edited rows still read correctly.
pub fn is_grid(model_params: &Value) -> bool {
    match model_params.get("is_xyz_grid") {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => s.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

/// Number of cells the axes imply, when every axis maps to a column. `None`
/// when at least one axis is of a type the matcher does not understand, since
/// then the count cannot be checked against reality.
pub fn expected_cells(axes: &[Axis]) -> Option<usize> {
    if axes.is_empty() || axes.iter().any(|a| a.column.is_none()) {
        return None;
    }
    Some(axes.iter().map(|a| a.values.len()).product())
}

/// Append `AND (col = v1 OR col = v2 …)` for one axis. `column` is always a
/// hard-coded `AxisColumn::as_str`, never user input; the values are bound.
fn push_any_of<'a, T>(qb: &mut QueryBuilder<'a, Postgres>, column: &str, values: Vec<T>)
where
    T: sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres> + Send + 'a,
{
    qb.push(" AND (");
    for (i, value) in values.into_iter().enumerate() {
        if i > 0 {
            qb.push(" OR ");
        }
        qb.push(column).push(" = ").push_bind(value);
    }
    qb.push(")");
}

/// Find the images a grid was most likely assembled from.
///
/// Returns them oldest-first; the caller orders them by axis position, which is
/// only knowable after the rows are in hand.
pub async fn find_members(
    pool: &PgPool,
    grid: &ImageRow,
    axes: &[Axis],
    window: Duration,
) -> Result<Vec<ImageRow>, sqlx::Error> {
    let lower = grid.created_at - window;
    let upper = grid.created_at + Duration::minutes(FORWARD_MARGIN_MINUTES);

    let mut qb = QueryBuilder::<Postgres>::new(
        "SELECT * FROM images WHERE deleted_at IS NULL \
         AND (model_params->>'is_xyz_grid' IS NULL \
              OR model_params->>'is_xyz_grid' <> 'true') \
         AND id <> ",
    );
    qb.push_bind(grid.id);
    qb.push(" AND created_at >= ").push_bind(lower);
    qb.push(" AND created_at <= ").push_bind(upper);

    let varied: Vec<AxisColumn> = axes.iter().filter_map(|a| a.column).collect();
    let holds_still = |c: AxisColumn| !varied.contains(&c);

    // Parameters the grid did not vary must be identical. Columns the grid has
    // no value for are left out entirely — an unknown cannot discriminate.
    if holds_still(AxisColumn::Seed)
        && let Some(v) = grid.seed
    {
        qb.push(" AND seed = ").push_bind(v);
    }
    if holds_still(AxisColumn::Steps)
        && let Some(v) = grid.steps
    {
        qb.push(" AND steps = ").push_bind(v);
    }
    if holds_still(AxisColumn::CfgScale)
        && let Some(v) = grid.cfg_scale
    {
        qb.push(" AND cfg_scale = ").push_bind(v);
    }
    if holds_still(AxisColumn::SamplerName)
        && let Some(v) = grid.sampler_name.as_deref()
    {
        qb.push(" AND sampler_name = ").push_bind(v.to_string());
    }
    if holds_still(AxisColumn::ModelName)
        && let Some(v) = grid.model_name.as_deref()
    {
        qb.push(" AND model_name = ").push_bind(v.to_string());
    }

    // Varied parameters must land on one of the axis values. Values that do not
    // parse for the column's type are dropped rather than failing the query —
    // a malformed axis should narrow less, not break the endpoint.
    for axis in axes {
        let Some(column) = axis.column.filter(|c| c.filterable()) else {
            continue;
        };
        match column {
            AxisColumn::Seed => {
                let values: Vec<i64> = axis
                    .values
                    .iter()
                    .filter_map(|v| v.trim().parse().ok())
                    .collect();
                if !values.is_empty() {
                    push_any_of(&mut qb, column.as_str(), values);
                }
            }
            AxisColumn::Steps => {
                let values: Vec<i32> = axis
                    .values
                    .iter()
                    .filter_map(|v| v.trim().parse().ok())
                    .collect();
                if !values.is_empty() {
                    push_any_of(&mut qb, column.as_str(), values);
                }
            }
            AxisColumn::CfgScale => {
                let values: Vec<rust_decimal::Decimal> = axis
                    .values
                    .iter()
                    .filter_map(|v| v.trim().parse().ok())
                    .collect();
                if !values.is_empty() {
                    push_any_of(&mut qb, column.as_str(), values);
                }
            }
            AxisColumn::SamplerName => {
                let values: Vec<String> = axis.values.iter().map(|v| v.trim().to_string()).collect();
                if !values.is_empty() {
                    push_any_of(&mut qb, column.as_str(), values);
                }
            }
            // Not filterable — reconciled in `place` instead.
            AxisColumn::ModelName => {}
        }
    }

    qb.push(" ORDER BY created_at ASC, id ASC LIMIT ");
    qb.push_bind(MAX_MEMBERS);
    qb.build_query_as::<ImageRow>().fetch_all(pool).await
}

/// Reduce a checkpoint name to something comparable across A1111's dropdown
/// title and the value stored in metadata: basename, no extension, no trailing
/// `[hash]`.
fn model_key(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let base = base.split('[').next().unwrap_or(base).trim();
    let base = base
        .strip_suffix(".safetensors")
        .or_else(|| base.strip_suffix(".ckpt"))
        .unwrap_or(base);
    base.trim().to_ascii_lowercase()
}

/// Whether a row's column equals an axis value. Numeric columns compare as
/// numbers, so the axis value `7` matches a stored `7.00`.
pub fn value_matches(row: &ImageRow, column: AxisColumn, candidate: &str) -> bool {
    let candidate = candidate.trim();
    match column {
        AxisColumn::Seed => {
            row.seed.is_some_and(|v| candidate.parse::<i64>() == Ok(v))
        }
        AxisColumn::Steps => {
            row.steps.is_some_and(|v| candidate.parse::<i32>() == Ok(v))
        }
        AxisColumn::CfgScale => {
            let Some(actual) = row.cfg_scale.and_then(|d| d.to_f64()) else {
                return false;
            };
            candidate
                .parse::<f64>()
                .is_ok_and(|want| (want - actual).abs() < 0.001)
        }
        AxisColumn::SamplerName => row
            .sampler_name
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case(candidate)),
        AxisColumn::ModelName => row
            .model_name
            .as_deref()
            .is_some_and(|s| model_key(s) == model_key(candidate)),
    }
}

/// Where a member sits in the montage: the index along each axis, plus the axis
/// value it matched. Slots are `[x, y, z]`; an axis the row could not be placed
/// on keeps index 0 and value `None`.
#[derive(Debug, Clone, Default)]
pub struct Placement {
    pub index: [usize; 3],
    pub values: [Option<String>; 3],
}

fn slot_of(axis_name: &str) -> usize {
    match axis_name {
        "x" => 0,
        "y" => 1,
        _ => 2,
    }
}

/// Locate one member on the grid's axes.
pub fn place(row: &ImageRow, axes: &[Axis]) -> Placement {
    let mut placement = Placement::default();
    for axis in axes {
        let Some(column) = axis.column else {
            continue;
        };
        let slot = slot_of(axis.name);
        if let Some(pos) = axis
            .values
            .iter()
            .position(|v| value_matches(row, column, v))
        {
            placement.index[slot] = pos;
            placement.values[slot] = Some(axis.values[pos].clone());
        }
    }
    placement
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn axis_types_map_to_columns_case_insensitively() {
        assert_eq!(axis_column("Seed"), Some(AxisColumn::Seed));
        assert_eq!(axis_column("steps"), Some(AxisColumn::Steps));
        assert_eq!(axis_column("CFG Scale"), Some(AxisColumn::CfgScale));
        assert_eq!(axis_column(" Sampler "), Some(AxisColumn::SamplerName));
        assert_eq!(axis_column("Checkpoint name"), Some(AxisColumn::ModelName));
        // Types the matcher has no column for stay axes, but map to nothing.
        assert_eq!(axis_column("Prompt S/R"), None);
        assert_eq!(axis_column("VAE"), None);
    }

    #[test]
    fn axis_values_split_on_commas_and_drop_quotes() {
        assert_eq!(parse_axis_values("5,7,9"), vec!["5", "7", "9"]);
        assert_eq!(
            parse_axis_values("\"Euler a\", \"DPM++ 2M Karras\""),
            vec!["Euler a", "DPM++ 2M Karras"]
        );
        assert!(parse_axis_values(" , ,").is_empty());
    }

    #[test]
    fn axes_are_read_from_model_params() {
        let mp = json!({
            "is_xyz_grid": true,
            "xyz_x_type": "CFG Scale",
            "xyz_x_values": "5,7,9",
            "xyz_y_type": "Sampler",
            "xyz_y_values": "Euler a, DPM++ 2M",
        });
        let axes = axes_of(&mp);
        assert_eq!(axes.len(), 2);
        assert_eq!(axes[0].name, "x");
        assert_eq!(axes[0].column, Some(AxisColumn::CfgScale));
        assert_eq!(axes[0].values.len(), 3);
        assert_eq!(axes[1].name, "y");
        assert_eq!(axes[1].column, Some(AxisColumn::SamplerName));
        assert_eq!(expected_cells(&axes), Some(6));
    }

    #[test]
    fn half_recorded_axes_are_ignored() {
        // A type with no values cannot be matched on, and counting it would
        // make expected_cells wrong.
        let mp = json!({ "xyz_x_type": "Seed" });
        assert!(axes_of(&mp).is_empty());
        let mp = json!({ "xyz_x_values": "1,2" });
        assert!(axes_of(&mp).is_empty());
    }

    #[test]
    fn expected_cells_is_unknown_when_an_axis_type_is_unsupported() {
        let mp = json!({
            "xyz_x_type": "Steps",
            "xyz_x_values": "20,30",
            "xyz_y_type": "Prompt S/R",
            "xyz_y_values": "cat, dog",
        });
        let axes = axes_of(&mp);
        assert_eq!(axes.len(), 2);
        assert_eq!(expected_cells(&axes), None);
    }

    #[test]
    fn grid_flag_reads_bool_and_string() {
        assert!(is_grid(&json!({ "is_xyz_grid": true })));
        assert!(is_grid(&json!({ "is_xyz_grid": "true" })));
        assert!(!is_grid(&json!({ "is_xyz_grid": false })));
        assert!(!is_grid(&json!({})));
    }

    #[test]
    fn checkpoint_names_compare_across_a1111_and_metadata_spellings() {
        // A1111's dropdown title vs. what lands in images.model_name.
        assert_eq!(model_key("SD\\animagine_v31.safetensors"), "animagine_v31");
        assert_eq!(model_key("animagine_v31.safetensors [a1b2c3d4]"), "animagine_v31");
        assert_eq!(model_key("models/pony.ckpt"), "pony");
        assert_ne!(model_key("pony_v6"), model_key("pony_v7"));
    }
}
