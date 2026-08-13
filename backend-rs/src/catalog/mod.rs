//! Model and LoRA catalog: usage statistics aggregated from the images table
//! (mirror endpoints/models.py and endpoints/loras.py). There are no dedicated
//! model/lora tables — everything is derived from `images.model_name` and the
//! `images.loras` JSONB array. CivitAI enrichment lives in the civitai module.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

use regex::Regex;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::dto::catalog::{
    LoraDetail, LoraListItem, LoraListResponse, ModelDetail, ModelListItem, ModelListResponse,
    ModelVersionStats, NamedStat,
};
use crate::util::round2;

/// (model_name, model_type, image_count, rated_count, avg_rating, high_rated_count)
type ModelStatRow = (String, Option<String>, i64, i64, Option<f64>, i64);
/// (lora_name, hash, image_count, rated_count, avg_rating, high_rated_count)
type LoraListRow = (String, Option<String>, i64, i64, Option<f64>, i64);
/// (name, count, avg_rating)
type NamedStatRow = (String, i64, Option<f64>);
/// (hash, image_count, rated_count, avg_rating, high_rated_count, avg_weight)
type LoraStatsRow = (Option<String>, i64, i64, Option<f64>, i64, Option<f64>);

fn version_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[_-]?[vV]\d+(\.\d+)?[a-zA-Z]*$").unwrap())
}

fn ext_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?i)\.(safetensors|ckpt|pt)$").unwrap())
}

/// Filename portion after the last `/` or `\` (mirror extract_display_name).
pub fn extract_display_name(full_name: &str) -> String {
    let name = full_name.replace('\\', "/");
    match name.rsplit_once('/') {
        Some((_, last)) => last.to_string(),
        None => name,
    }
}

/// Strip a known model-file extension (.safetensors/.ckpt/.pt). Used before a
/// CivitAI name search (mirror the regex in the Python civitai endpoints).
pub fn strip_extension(name: &str) -> String {
    ext_re().replace(name, "").to_string()
}

/// Strip a trailing version suffix while keeping the extension
/// (mirror extract_base_model_name): animagine_v80.safetensors -> animagine.safetensors.
pub fn extract_base_model_name(display_name: &str) -> String {
    let ext = ext_re()
        .find(display_name)
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let without_ext = &display_name[..display_name.len() - ext.len()];
    let replaced = version_re().replace(without_ext, "");
    let base: &str = if replaced.is_empty() {
        without_ext
    } else {
        replaced.as_ref()
    };
    format!("{base}{ext}")
}

fn named_stats(rows: Vec<NamedStatRow>) -> Vec<NamedStat> {
    rows.into_iter()
        .map(|(name, count, avg)| NamedStat {
            name,
            count,
            avg_rating: avg.map(round2),
        })
        .collect()
}

fn zero_distribution() -> BTreeMap<i32, i64> {
    (0..=5).map(|r| (r, 0)).collect()
}

// ----------------------------------------------------------------------------
// Models
// ----------------------------------------------------------------------------

#[derive(Default)]
struct ModelGroup {
    version_count: i64,
    model_type: Option<String>,
    image_count: i64,
    rated_count: i64,
    rating_sum: f64,
    rating_count: i64,
    high_rated_count: i64,
}

struct ModelEntry {
    base_name: String,
    model_type: Option<String>,
    image_count: i64,
    rated_count: i64,
    avg_rating: Option<f64>,
    high_rated_count: i64,
    version_count: i64,
}

/// Model list grouped by base name, with Python-side filtering, sorting, and
/// pagination (mirror endpoints/models.py:get_models).
#[allow(clippy::too_many_arguments)]
pub async fn models_list(
    pool: &PgPool,
    q: Option<&str>,
    model_type: Option<&str>,
    min_count: i64,
    min_rating: Option<f64>,
    sort_by: &str,
    sort_order: &str,
    limit: i64,
    offset: i64,
) -> Result<ModelListResponse, sqlx::Error> {
    let rows: Vec<ModelStatRow> = sqlx::query_as(
        "SELECT model_name, model_type, count(*), \
                count(*) FILTER (WHERE rating > 0), \
                avg(CASE WHEN rating > 0 THEN rating END)::float8, \
                count(*) FILTER (WHERE rating >= 4) \
         FROM images \
         WHERE deleted_at IS NULL AND model_name IS NOT NULL \
             AND ($1::text IS NULL OR model_type = $1) \
         GROUP BY model_name, model_type",
    )
    .bind(model_type)
    .fetch_all(pool)
    .await?;

    // Group versions by base name, preserving first-seen order for stable ties.
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, ModelGroup> = HashMap::new();
    for (model_name, mtype, image_count, rated_count, avg_rating, high_rated_count) in rows {
        let base = extract_base_model_name(&extract_display_name(&model_name));
        if !groups.contains_key(&base) {
            order.push(base.clone());
        }
        let g = groups.entry(base).or_default();
        g.version_count += 1;
        if g.model_type.as_deref().unwrap_or("").is_empty() {
            g.model_type = mtype;
        }
        g.image_count += image_count;
        g.rated_count += rated_count;
        g.high_rated_count += high_rated_count;
        if let Some(avg) = avg_rating {
            g.rating_sum += avg * rated_count as f64;
            g.rating_count += rated_count;
        }
    }

    let q_lower = q.filter(|s| !s.is_empty()).map(str::to_lowercase);

    let mut entries: Vec<ModelEntry> = Vec::new();
    for base in order {
        let g = &groups[&base];
        let avg_rating = if g.rating_count > 0 {
            Some(g.rating_sum / g.rating_count as f64)
        } else {
            None
        };
        if g.image_count < min_count {
            continue;
        }
        if let Some(mr) = min_rating
            && avg_rating.is_none_or(|a| a < mr)
        {
            continue;
        }
        if let Some(ref ql) = q_lower
            && !base.to_lowercase().contains(ql)
        {
            continue;
        }
        entries.push(ModelEntry {
            model_type: g.model_type.clone(),
            image_count: g.image_count,
            rated_count: g.rated_count,
            avg_rating: avg_rating.map(round2),
            high_rated_count: g.high_rated_count,
            version_count: g.version_count,
            base_name: base,
        });
    }

    let desc = sort_order != "asc";
    entries.sort_by(|a, b| {
        let ord = match sort_by {
            "rating" => a
                .avg_rating
                .unwrap_or(0.0)
                .partial_cmp(&b.avg_rating.unwrap_or(0.0))
                .unwrap_or(Ordering::Equal)
                .then(a.image_count.cmp(&b.image_count)),
            "name" => a.base_name.to_lowercase().cmp(&b.base_name.to_lowercase()),
            _ => a.image_count.cmp(&b.image_count),
        };
        if desc { ord.reverse() } else { ord }
    });

    let total = entries.len() as i64;
    let items: Vec<ModelListItem> = entries
        .into_iter()
        .skip(offset.max(0) as usize)
        .take(limit.max(0) as usize)
        .map(|e| ModelListItem {
            display_name: e.base_name.clone(),
            name: e.base_name,
            model_type: e.model_type,
            image_count: e.image_count,
            rated_count: e.rated_count,
            avg_rating: e.avg_rating,
            high_rated_count: e.high_rated_count,
            version_count: e.version_count,
        })
        .collect();

    Ok(ModelListResponse { items, total })
}

/// Aggregated detail for a base model: per-version stats, rating histograms,
/// top samplers and loras (mirror endpoints/models.py:get_model_detail).
pub async fn model_detail(pool: &PgPool, base_name: &str) -> Result<ModelDetail, sqlx::Error> {
    let all_names: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT model_name FROM images \
         WHERE deleted_at IS NULL AND model_name IS NOT NULL",
    )
    .fetch_all(pool)
    .await?;

    let versions_filter: Vec<String> = all_names
        .into_iter()
        .filter(|n| extract_base_model_name(&extract_display_name(n)) == base_name)
        .collect();

    if versions_filter.is_empty() {
        return Ok(ModelDetail {
            name: base_name.to_string(),
            display_name: base_name.to_string(),
            model_type: None,
            image_count: 0,
            rated_count: 0,
            avg_rating: None,
            high_rated_count: 0,
            rating_distribution: zero_distribution(),
            top_samplers: Vec::new(),
            top_loras: Vec::new(),
            versions: Vec::new(),
        });
    }

    let version_rows: Vec<ModelStatRow> = sqlx::query_as(
        "SELECT model_name, model_type, count(*), \
                count(*) FILTER (WHERE rating > 0), \
                avg(CASE WHEN rating > 0 THEN rating END)::float8, \
                count(*) FILTER (WHERE rating >= 4) \
         FROM images WHERE deleted_at IS NULL AND model_name = ANY($1) \
         GROUP BY model_name, model_type ORDER BY count(*) DESC",
    )
    .bind(&versions_filter)
    .fetch_all(pool)
    .await?;

    // Per-version rating histograms in a single query.
    let dist_rows: Vec<(String, i16, i64)> = sqlx::query_as(
        "SELECT model_name, rating, count(*) FROM images \
         WHERE deleted_at IS NULL AND model_name = ANY($1) GROUP BY model_name, rating",
    )
    .bind(&versions_filter)
    .fetch_all(pool)
    .await?;
    let mut dist_by_version: HashMap<String, BTreeMap<i32, i64>> = HashMap::new();
    for (name, rating, count) in dist_rows {
        dist_by_version
            .entry(name)
            .or_insert_with(zero_distribution)
            .insert(rating as i32, count);
    }

    let mut model_type: Option<String> = None;
    let mut versions: Vec<ModelVersionStats> = Vec::new();
    for (name, mtype, image_count, rated_count, avg_rating, high_rated_count) in version_rows {
        if model_type.as_deref().unwrap_or("").is_empty() {
            model_type = mtype;
        }
        let dist = dist_by_version
            .remove(&name)
            .unwrap_or_else(zero_distribution);
        versions.push(ModelVersionStats {
            display_name: extract_display_name(&name),
            name,
            image_count,
            rated_count,
            avg_rating: avg_rating.map(round2),
            high_rated_count,
            rating_distribution: dist,
        });
    }

    let total_image_count: i64 = versions.iter().map(|v| v.image_count).sum();
    let total_rated_count: i64 = versions.iter().map(|v| v.rated_count).sum();
    let total_high_rated_count: i64 = versions.iter().map(|v| v.high_rated_count).sum();
    let rating_sum: f64 = versions
        .iter()
        .map(|v| v.avg_rating.unwrap_or(0.0) * v.rated_count as f64)
        .sum();
    let avg_rating = if total_rated_count > 0 {
        Some(round2(rating_sum / total_rated_count as f64))
    } else {
        None
    };

    let mut rating_distribution = zero_distribution();
    for v in &versions {
        for (rating, count) in &v.rating_distribution {
            *rating_distribution.entry(*rating).or_insert(0) += count;
        }
    }

    let top_samplers = named_stats(
        sqlx::query_as(
            "SELECT sampler_name, count(*), avg(CASE WHEN rating > 0 THEN rating END)::float8 \
             FROM images WHERE deleted_at IS NULL AND model_name = ANY($1) \
                 AND sampler_name IS NOT NULL \
             GROUP BY sampler_name ORDER BY count(*) DESC LIMIT 10",
        )
        .bind(&versions_filter)
        .fetch_all(pool)
        .await?,
    );

    let lora_rows: Vec<(Option<String>, i64, Option<f64>)> = sqlx::query_as(
        "SELECT name, count(*), avg(CASE WHEN rating > 0 THEN rating END)::float8 \
         FROM (\
            SELECT rating, jsonb_array_elements(loras)->>'name' AS name FROM images \
            WHERE deleted_at IS NULL AND model_name = ANY($1) AND jsonb_array_length(loras) > 0\
         ) sub \
         GROUP BY name ORDER BY count(*) DESC LIMIT 10",
    )
    .bind(&versions_filter)
    .fetch_all(pool)
    .await?;
    let top_loras: Vec<NamedStat> = lora_rows
        .into_iter()
        .filter_map(|(name, count, avg)| {
            name.map(|name| NamedStat {
                name,
                count,
                avg_rating: avg.map(round2),
            })
        })
        .collect();

    Ok(ModelDetail {
        name: base_name.to_string(),
        display_name: base_name.to_string(),
        model_type,
        image_count: total_image_count,
        rated_count: total_rated_count,
        avg_rating,
        high_rated_count: total_high_rated_count,
        rating_distribution,
        top_samplers,
        top_loras,
        versions,
    })
}

// ----------------------------------------------------------------------------
// LoRAs
// ----------------------------------------------------------------------------

/// Derived table of one row per (image, lora) with a non-null lora name.
const LORA_DATA: &str = "(SELECT v.rating, v.lora_name, v.lora_hash FROM (\
    SELECT rating, lo->>'name' AS lora_name, lo->>'hash' AS lora_hash FROM (\
        SELECT rating, jsonb_array_elements(loras) AS lo FROM images \
        WHERE deleted_at IS NULL AND jsonb_array_length(loras) > 0\
    ) e\
) v WHERE v.lora_name IS NOT NULL) lora_data";

fn push_lora_core(
    qb: &mut QueryBuilder<'_, Postgres>,
    min_count: i64,
    q: Option<&str>,
    min_rating: Option<f64>,
) {
    qb.push(" FROM ")
        .push(LORA_DATA)
        .push(" GROUP BY lora_name HAVING count(*) >= ")
        .push_bind(min_count);
    if let Some(q) = q.filter(|s| !s.is_empty()) {
        qb.push(" AND lower(lora_name) LIKE ")
            .push_bind(format!("%{}%", q.to_lowercase()));
    }
    if let Some(mr) = min_rating {
        qb.push(" AND avg(CASE WHEN rating > 0 THEN rating END) >= ")
            .push_bind(mr);
    }
}

/// LoRA list with usage statistics (mirror endpoints/loras.py:get_loras).
#[allow(clippy::too_many_arguments)]
pub async fn loras_list(
    pool: &PgPool,
    q: Option<&str>,
    min_count: i64,
    min_rating: Option<f64>,
    sort_by: &str,
    sort_order: &str,
    limit: i64,
    offset: i64,
) -> Result<LoraListResponse, sqlx::Error> {
    let mut cqb = QueryBuilder::<Postgres>::new("SELECT count(*) FROM (SELECT lora_name");
    push_lora_core(&mut cqb, min_count, q, min_rating);
    cqb.push(") sub");
    let total: i64 = cqb.build_query_scalar().fetch_one(pool).await?;

    let mut pqb = QueryBuilder::<Postgres>::new(
        "SELECT lora_name, max(lora_hash), count(*), \
                count(*) FILTER (WHERE rating > 0), \
                avg(CASE WHEN rating > 0 THEN rating END)::float8, \
                count(*) FILTER (WHERE rating >= 4)",
    );
    push_lora_core(&mut pqb, min_count, q, min_rating);
    pqb.push(" ORDER BY ");
    match sort_by {
        "rating" => {
            pqb.push("avg(CASE WHEN rating > 0 THEN rating END)");
        }
        "name" => {
            // Sort by filename only (strip any path), matching the Python regexp.
            pqb.push("lower(regexp_replace(lora_name, ")
                .push_bind(r"^.*[/\\]")
                .push(", '', 'g'))");
        }
        _ => {
            pqb.push("count(*)");
        }
    }
    pqb.push(if sort_order == "asc" {
        " ASC NULLS LAST"
    } else {
        " DESC NULLS LAST"
    });
    pqb.push(" LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let rows: Vec<LoraListRow> = pqb.build_query_as().fetch_all(pool).await?;
    let items = rows
        .into_iter()
        .map(
            |(name, hash, image_count, rated_count, avg_rating, high_rated_count)| LoraListItem {
                display_name: extract_display_name(&name),
                name,
                hash,
                image_count,
                rated_count,
                avg_rating: avg_rating.map(round2),
                high_rated_count,
            },
        )
        .collect();

    Ok(LoraListResponse { items, total })
}

/// Look up any non-null hash recorded for a LoRA name (for CivitAI by-hash
/// lookup). Mirrors the hash query in endpoints/loras.py:get_lora_civitai_info.
pub async fn lora_hash(pool: &PgPool, lora_name: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT lo->>'hash' FROM (\
            SELECT jsonb_array_elements(loras) AS lo FROM images \
            WHERE deleted_at IS NULL AND jsonb_array_length(loras) > 0\
         ) u \
         WHERE lo->>'name' = $1 AND lo->>'hash' IS NOT NULL LIMIT 1",
    )
    .bind(lora_name)
    .fetch_optional(pool)
    .await
}

/// Detailed statistics for one LoRA (mirror endpoints/loras.py:get_lora_detail).
pub async fn lora_detail(pool: &PgPool, lora_name: &str) -> Result<LoraDetail, sqlx::Error> {
    let filtered = "(SELECT rating, model_name, sampler_name, lora_hash, lora_weight, lora_name \
        FROM (\
            SELECT rating, model_name, sampler_name, lo->>'hash' AS lora_hash, \
                   (lo->>'weight')::numeric AS lora_weight, lo->>'name' AS lora_name \
            FROM (\
                SELECT rating, model_name, sampler_name, jsonb_array_elements(loras) AS lo \
                FROM images WHERE deleted_at IS NULL AND jsonb_array_length(loras) > 0\
            ) e\
        ) v WHERE v.lora_name = $1) lf";

    let stats: Option<LoraStatsRow> = sqlx::query_as(
        &format!(
            "SELECT max(lora_hash), count(*), count(*) FILTER (WHERE rating > 0), \
                    avg(CASE WHEN rating > 0 THEN rating END)::float8, \
                    count(*) FILTER (WHERE rating >= 4), avg(lora_weight)::float8 \
             FROM {filtered}"
        ),
    )
    .bind(lora_name)
    .fetch_optional(pool)
    .await?;

    let (hash, image_count, rated_count, avg_rating, high_rated_count, avg_weight) =
        match stats {
            Some(s) if s.1 > 0 => s,
            _ => {
                return Ok(LoraDetail {
                    name: lora_name.to_string(),
                    display_name: extract_display_name(lora_name),
                    hash: None,
                    image_count: 0,
                    rated_count: 0,
                    avg_rating: None,
                    high_rated_count: 0,
                    rating_distribution: zero_distribution(),
                    avg_weight: None,
                    top_models: Vec::new(),
                    top_samplers: Vec::new(),
                });
            }
        };

    let dist_rows: Vec<(i16, i64)> =
        sqlx::query_as(&format!("SELECT rating, count(*) FROM {filtered} GROUP BY rating"))
            .bind(lora_name)
            .fetch_all(pool)
            .await?;
    let mut rating_distribution = zero_distribution();
    for (rating, count) in dist_rows {
        rating_distribution.insert(rating as i32, count);
    }

    let top_models = named_stats(
        sqlx::query_as(&format!(
            "SELECT model_name, count(*), avg(CASE WHEN rating > 0 THEN rating END)::float8 \
             FROM {filtered} WHERE model_name IS NOT NULL \
             GROUP BY model_name ORDER BY count(*) DESC LIMIT 10"
        ))
        .bind(lora_name)
        .fetch_all(pool)
        .await?,
    );

    let top_samplers = named_stats(
        sqlx::query_as(&format!(
            "SELECT sampler_name, count(*), avg(CASE WHEN rating > 0 THEN rating END)::float8 \
             FROM {filtered} WHERE sampler_name IS NOT NULL \
             GROUP BY sampler_name ORDER BY count(*) DESC LIMIT 10"
        ))
        .bind(lora_name)
        .fetch_all(pool)
        .await?,
    );

    Ok(LoraDetail {
        name: lora_name.to_string(),
        display_name: extract_display_name(lora_name),
        hash,
        image_count,
        rated_count,
        avg_rating: avg_rating.map(round2),
        high_rated_count,
        rating_distribution,
        avg_weight: avg_weight.map(round2),
        top_models,
        top_samplers,
    })
}
