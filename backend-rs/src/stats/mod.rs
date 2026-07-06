//! Statistics aggregation over the images table (mirror endpoints/stats.py).
//!
//! Python's `func.count(case(...))` becomes `count(*) FILTER (WHERE ...)`, and
//! `func.avg(case((rating > 0, rating)))` becomes
//! `avg(CASE WHEN rating > 0 THEN rating END)`. Averages are cast to float8 so
//! they map to f64, then rounded to 2 decimals to match Python's `round(x, 2)`.
//!
//! NOTE: the Python endpoints cache results in-memory (10 min TTL). That is a
//! transparent optimization; results here are computed fresh on each request.

use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::dto::stats::{
    CountItem, ModelRatingDistributionItem, RatingAnalysisItem, RatingAnalysisResponse,
    RatingDistribution, StatsOverview, StatsResponse, TimeSeriesItem,
};
use crate::util::round2;

/// Raw row for the per-model rating histogram:
/// (model_name, rating_0..=rating_5, total, avg_rating).
type ModelRatingRow = (String, i64, i64, i64, i64, i64, i64, i64, Option<f64>);

fn to_count_items(rows: Vec<(String, i64)>) -> Vec<CountItem> {
    rows.into_iter()
        .map(|(name, count)| CountItem { name, count })
        .collect()
}

/// Map rating-analysis rows, rounding the average and dropping null group keys
/// (e.g. a LoRA element without a name).
fn to_analysis_items(rows: Vec<(Option<String>, f64, i64, i64)>) -> Vec<RatingAnalysisItem> {
    rows.into_iter()
        .filter_map(|(name, avg, count, high)| {
            name.map(|name| RatingAnalysisItem {
                name,
                avg_rating: round2(avg),
                count,
                high_rated_count: high,
            })
        })
        .collect()
}

/// Library overview + breakdowns + daily import/update time series.
pub async fn get_stats(pool: &PgPool, days: i64) -> Result<StatsResponse, sqlx::Error> {
    let (total, favorites, rated, unrated, avg): (i64, i64, i64, i64, Option<f64>) =
        sqlx::query_as(
            "SELECT count(*), \
                    count(*) FILTER (WHERE is_favorite), \
                    count(*) FILTER (WHERE rating > 0), \
                    count(*) FILTER (WHERE rating = 0), \
                    avg(CASE WHEN rating > 0 THEN rating END)::float8 \
             FROM images WHERE deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await?;
    let overview = StatsOverview {
        total_images: total,
        total_favorites: favorites,
        total_rated: rated,
        total_unrated: unrated,
        avg_rating: avg.map(round2),
    };

    let by_model_type = to_count_items(
        sqlx::query_as(
            "SELECT model_type, count(*) FROM images \
             WHERE deleted_at IS NULL AND model_type IS NOT NULL \
             GROUP BY model_type ORDER BY count(*) DESC LIMIT 10",
        )
        .fetch_all(pool)
        .await?,
    );

    let by_source_tool = to_count_items(
        sqlx::query_as(
            "SELECT source_tool, count(*) FROM images WHERE deleted_at IS NULL \
             GROUP BY source_tool ORDER BY count(*) DESC",
        )
        .fetch_all(pool)
        .await?,
    );

    let by_model_name = to_count_items(
        sqlx::query_as(
            "SELECT model_name, count(*) FROM images \
             WHERE deleted_at IS NULL AND model_name IS NOT NULL \
             GROUP BY model_name ORDER BY count(*) DESC LIMIT 10",
        )
        .fetch_all(pool)
        .await?,
    );

    let by_sampler = to_count_items(
        sqlx::query_as(
            "SELECT sampler_name, count(*) FROM images \
             WHERE deleted_at IS NULL AND sampler_name IS NOT NULL \
             GROUP BY sampler_name ORDER BY count(*) DESC LIMIT 10",
        )
        .fetch_all(pool)
        .await?,
    );

    let lora_rows: Vec<(Option<String>, i64)> = sqlx::query_as(
        "SELECT name, count(*) FROM (\
            SELECT jsonb_array_elements(loras)->>'name' AS name FROM images \
            WHERE deleted_at IS NULL AND jsonb_array_length(loras) > 0\
         ) sub GROUP BY name ORDER BY count(*) DESC LIMIT 10",
    )
    .fetch_all(pool)
    .await?;
    let by_lora: Vec<CountItem> = lora_rows
        .into_iter()
        .filter_map(|(name, count)| name.map(|name| CountItem { name, count }))
        .collect();

    let rating_rows: Vec<(i16, i64)> = sqlx::query_as(
        "SELECT rating, count(*) FROM images WHERE deleted_at IS NULL \
         GROUP BY rating ORDER BY rating",
    )
    .fetch_all(pool)
    .await?;
    let by_rating: Vec<RatingDistribution> = rating_rows
        .into_iter()
        .map(|(rating, count)| RatingDistribution {
            rating: rating as i32,
            count,
        })
        .collect();

    let start = Utc::now() - Duration::days(days);

    let daily_counts = to_time_series(
        sqlx::query_as(
            "SELECT to_char(date_trunc('day', created_at), 'YYYY-MM-DD'), count(*) \
             FROM images WHERE deleted_at IS NULL AND created_at >= $1 \
             GROUP BY date_trunc('day', created_at) ORDER BY date_trunc('day', created_at)",
        )
        .bind(start)
        .fetch_all(pool)
        .await?,
    );

    let daily_updates = to_time_series(
        sqlx::query_as(
            "SELECT to_char(date_trunc('day', updated_at), 'YYYY-MM-DD'), count(*) \
             FROM images WHERE deleted_at IS NULL AND updated_at >= $1 \
                 AND updated_at > created_at \
             GROUP BY date_trunc('day', updated_at) ORDER BY date_trunc('day', updated_at)",
        )
        .bind(start)
        .fetch_all(pool)
        .await?,
    );

    Ok(StatsResponse {
        overview,
        by_model_type,
        by_source_tool,
        by_model_name,
        by_sampler,
        by_lora,
        by_rating,
        daily_counts,
        daily_updates,
    })
}

fn to_time_series(rows: Vec<(String, i64)>) -> Vec<TimeSeriesItem> {
    rows.into_iter()
        .map(|(date, count)| TimeSeriesItem { date, count })
        .collect()
}

/// Models with at least `min_count` rated images, ordered by average rating.
pub async fn models_for_analysis(
    pool: &PgPool,
    min_count: i64,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT model_name FROM images \
         WHERE deleted_at IS NULL AND rating > 0 AND model_name IS NOT NULL \
         GROUP BY model_name HAVING count(*) >= $1 ORDER BY avg(rating) DESC",
    )
    .bind(min_count)
    .fetch_all(pool)
    .await
}

/// LoRA names used at least `min_count` times, most-used first.
pub async fn loras_for_filter(pool: &PgPool, min_count: i64) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<Option<String>> = sqlx::query_scalar(
        "SELECT name FROM (\
            SELECT jsonb_array_elements(loras)->>'name' AS name FROM images \
            WHERE deleted_at IS NULL AND jsonb_array_length(loras) > 0\
         ) sub GROUP BY name HAVING count(*) >= $1 ORDER BY count(*) DESC",
    )
    .bind(min_count)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().flatten().collect())
}

/// Sampler names used at least `min_count` times, most-used first.
pub async fn samplers_for_filter(
    pool: &PgPool,
    min_count: i64,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT sampler_name FROM images \
         WHERE deleted_at IS NULL AND sampler_name IS NOT NULL \
         GROUP BY sampler_name HAVING count(*) >= $1 ORDER BY count(*) DESC",
    )
    .bind(min_count)
    .fetch_all(pool)
    .await
}

/// Which settings correlate with higher ratings. When `model_name` is given,
/// the by_model breakdown is skipped and the others are restricted to it.
pub async fn rating_analysis(
    pool: &PgPool,
    min_count: i64,
    model_name: Option<&str>,
) -> Result<RatingAnalysisResponse, sqlx::Error> {
    let by_model = if model_name.is_none() {
        let rows: Vec<(Option<String>, f64, i64, i64)> = sqlx::query_as(
            "SELECT model_name, avg(rating)::float8, count(*), \
                    count(*) FILTER (WHERE rating >= 4) \
             FROM images WHERE deleted_at IS NULL AND rating > 0 AND model_name IS NOT NULL \
             GROUP BY model_name HAVING count(*) >= $1 ORDER BY avg(rating) DESC LIMIT 10",
        )
        .bind(min_count)
        .fetch_all(pool)
        .await?;
        to_analysis_items(rows)
    } else {
        Vec::new()
    };

    // `$1::text IS NULL OR model_name = $1` applies the optional model filter
    // with a single bound parameter.
    let by_sampler = to_analysis_items(
        sqlx::query_as(
            "SELECT sampler_name, avg(rating)::float8, count(*), \
                    count(*) FILTER (WHERE rating >= 4) \
             FROM images \
             WHERE deleted_at IS NULL AND rating > 0 \
                 AND ($1::text IS NULL OR model_name = $1) AND sampler_name IS NOT NULL \
             GROUP BY sampler_name HAVING count(*) >= $2 ORDER BY avg(rating) DESC LIMIT 10",
        )
        .bind(model_name)
        .bind(min_count)
        .fetch_all(pool)
        .await?,
    );

    let by_lora = to_analysis_items(
        sqlx::query_as(
            "SELECT name, avg(rating)::float8, count(*), \
                    count(*) FILTER (WHERE rating >= 4) \
             FROM (\
                SELECT rating, jsonb_array_elements(loras)->>'name' AS name FROM images \
                WHERE deleted_at IS NULL AND rating > 0 \
                    AND ($1::text IS NULL OR model_name = $1) AND jsonb_array_length(loras) > 0\
             ) sub \
             GROUP BY name HAVING count(*) >= $2 ORDER BY avg(rating) DESC LIMIT 10",
        )
        .bind(model_name)
        .bind(min_count)
        .fetch_all(pool)
        .await?,
    );

    let by_steps = to_analysis_items(
        sqlx::query_as(
            "SELECT steps_range, avg(rating)::float8, count(*), \
                    count(*) FILTER (WHERE rating >= 4) \
             FROM (\
                SELECT rating, CASE \
                    WHEN steps < 20 THEN '< 20' WHEN steps < 30 THEN '20-29' \
                    WHEN steps < 40 THEN '30-39' WHEN steps < 50 THEN '40-49' \
                    ELSE '50+' END AS steps_range \
                FROM images WHERE deleted_at IS NULL AND rating > 0 \
                    AND ($1::text IS NULL OR model_name = $1) AND steps IS NOT NULL\
             ) sub \
             GROUP BY steps_range HAVING count(*) >= $2 ORDER BY avg(rating) DESC",
        )
        .bind(model_name)
        .bind(min_count)
        .fetch_all(pool)
        .await?,
    );

    let by_cfg = to_analysis_items(
        sqlx::query_as(
            "SELECT cfg_range, avg(rating)::float8, count(*), \
                    count(*) FILTER (WHERE rating >= 4) \
             FROM (\
                SELECT rating, CASE \
                    WHEN cfg_scale < 5 THEN '< 5' WHEN cfg_scale < 7 THEN '5-6.9' \
                    WHEN cfg_scale < 9 THEN '7-8.9' WHEN cfg_scale < 12 THEN '9-11.9' \
                    ELSE '12+' END AS cfg_range \
                FROM images WHERE deleted_at IS NULL AND rating > 0 \
                    AND ($1::text IS NULL OR model_name = $1) AND cfg_scale IS NOT NULL\
             ) sub \
             GROUP BY cfg_range HAVING count(*) >= $2 ORDER BY avg(rating) DESC",
        )
        .bind(model_name)
        .bind(min_count)
        .fetch_all(pool)
        .await?,
    );

    Ok(RatingAnalysisResponse {
        by_model,
        by_sampler,
        by_lora,
        by_steps,
        by_cfg,
        filtered_by_model: model_name.map(str::to_string),
    })
}

/// Per-model histogram of ratings 0..5 with totals and average.
pub async fn model_rating_distribution(
    pool: &PgPool,
    min_count: i64,
    limit: i64,
) -> Result<Vec<ModelRatingDistributionItem>, sqlx::Error> {
    let rows: Vec<ModelRatingRow> = sqlx::query_as(
        "SELECT model_name, \
                count(*) FILTER (WHERE rating = 0), \
                count(*) FILTER (WHERE rating = 1), \
                count(*) FILTER (WHERE rating = 2), \
                count(*) FILTER (WHERE rating = 3), \
                count(*) FILTER (WHERE rating = 4), \
                count(*) FILTER (WHERE rating = 5), \
                count(*), \
                avg(CASE WHEN rating > 0 THEN rating END)::float8 \
         FROM images WHERE deleted_at IS NULL AND model_name IS NOT NULL \
         GROUP BY model_name HAVING count(*) >= $1 ORDER BY count(*) DESC LIMIT $2",
    )
    .bind(min_count)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(model_name, r0, r1, r2, r3, r4, r5, total, avg)| ModelRatingDistributionItem {
                model_name,
                rating_0: r0,
                rating_1: r1,
                rating_2: r2,
                rating_3: r3,
                rating_4: r4,
                rating_5: r5,
                total,
                avg_rating: avg.map(round2),
            },
        )
        .collect())
}
