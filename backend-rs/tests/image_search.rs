//! Integration tests for the image search query builder (`image::push_filters`
//! / `sort_clause` / pagination), which is the single most-used SQL in the
//! backend and was previously untested.

mod common;

use common::{insert_image, insert_sequence, utc, NewImage};
use promptbox::image::{self, SearchParams};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Run a search and return the ids in result order.
async fn ids(pool: &PgPool, p: &SearchParams) -> Vec<Uuid> {
    image::list(pool, p)
        .await
        .expect("list")
        .items
        .into_iter()
        .map(|r| r.id)
        .collect()
}

/// Run a search and return the total count (which uses the same filters).
async fn total(pool: &PgPool, p: &SearchParams) -> i64 {
    image::list(pool, p).await.expect("list").total
}

// ---------------------------------------------------------------------------
// deleted_at handling
// ---------------------------------------------------------------------------

/// `include_deleted` does NOT mean "also include deleted": it switches the
/// listing to *only* deleted images (the trash view). Documented in docs/13 as
/// a naming/behaviour mismatch — pinned here so a rename cannot change the
/// semantics silently.
#[sqlx::test(migrations = "./migrations")]
async fn include_deleted_returns_only_deleted(pool: PgPool) {
    let live = insert_image(&pool, NewImage::default()).await;
    let trashed = insert_image(
        &pool,
        NewImage {
            deleted_at: Some(utc(2026, 2, 1, 0, 0, 0)),
            ..Default::default()
        },
    )
    .await;

    let default_view = ids(&pool, &SearchParams::default()).await;
    assert_eq!(default_view, vec![live], "default view hides deleted images");

    let trash_view = ids(
        &pool,
        &SearchParams {
            include_deleted: true,
            ..Default::default()
        },
    )
    .await;
    assert_eq!(
        trash_view,
        vec![trashed],
        "include_deleted returns ONLY deleted images, not live + deleted"
    );
}

// ---------------------------------------------------------------------------
// scalar equality filters
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn source_tool_and_model_type_filter_exactly(pool: PgPool) {
    let comfy = insert_image(&pool, NewImage::default()).await;
    let a1111 = insert_image(
        &pool,
        NewImage {
            source_tool: "a1111".to_string(),
            model_type: Some("sd15".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                source_tool: Some("a1111".to_string()),
                ..Default::default()
            }
        )
        .await,
        vec![a1111]
    );
    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                model_type: Some("sdxl".to_string()),
                ..Default::default()
            }
        )
        .await,
        vec![comfy]
    );
}

/// `exact_rating` wins over `min_rating` (mirrors the Python service), and
/// `max_rating` is applied independently of both.
#[sqlx::test(migrations = "./migrations")]
async fn rating_filters_precedence(pool: PgPool) {
    for r in 0..=5i16 {
        insert_image(
            &pool,
            NewImage {
                rating: r,
                original_filename: format!("r{r}.png"),
                ..Default::default()
            },
        )
        .await;
    }

    assert_eq!(
        total(
            &pool,
            &SearchParams {
                min_rating: Some(4),
                ..Default::default()
            }
        )
        .await,
        2,
        "min_rating=4 matches 4 and 5"
    );
    assert_eq!(
        total(
            &pool,
            &SearchParams {
                exact_rating: Some(3),
                min_rating: Some(5),
                ..Default::default()
            }
        )
        .await,
        1,
        "exact_rating takes precedence over min_rating"
    );
    assert_eq!(
        total(
            &pool,
            &SearchParams {
                min_rating: Some(2),
                max_rating: Some(3),
                ..Default::default()
            }
        )
        .await,
        2,
        "min_rating and max_rating combine into a range"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn boolean_flag_filters(pool: PgPool) {
    let fav = insert_image(
        &pool,
        NewImage {
            is_favorite: true,
            ..Default::default()
        },
    )
    .await;
    let needs = insert_image(
        &pool,
        NewImage {
            needs_improvement: true,
            ..Default::default()
        },
    )
    .await;
    insert_image(&pool, NewImage::default()).await;

    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                is_favorite: Some(true),
                ..Default::default()
            }
        )
        .await,
        vec![fav]
    );
    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                needs_improvement: Some(true),
                ..Default::default()
            }
        )
        .await,
        vec![needs]
    );
    assert_eq!(
        total(
            &pool,
            &SearchParams {
                is_favorite: Some(false),
                ..Default::default()
            }
        )
        .await,
        2,
        "is_favorite=false is an explicit filter, not 'unset'"
    );
}

// ---------------------------------------------------------------------------
// text-ish filters
// ---------------------------------------------------------------------------

/// model_name is a case-insensitive substring match, and LIKE wildcards in the
/// user's input are escaped (so `%` matches a literal percent sign).
#[sqlx::test(migrations = "./migrations")]
async fn model_name_is_escaped_substring_match(pool: PgPool) {
    let pony = insert_image(
        &pool,
        NewImage {
            model_name: Some("PonyDiffusionV6XL".to_string()),
            ..Default::default()
        },
    )
    .await;
    let percent = insert_image(
        &pool,
        NewImage {
            model_name: Some("100%_real".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                model_name: Some("ponydiffusion".to_string()),
                ..Default::default()
            }
        )
        .await,
        vec![pony],
        "match is case-insensitive and partial"
    );
    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                model_name: Some("100%_r".to_string()),
                ..Default::default()
            }
        )
        .await,
        vec![percent],
        "% and _ from user input are escaped, not treated as wildcards"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn sampler_name_is_exact(pool: PgPool) {
    let euler = insert_image(
        &pool,
        NewImage {
            sampler_name: Some("euler".to_string()),
            ..Default::default()
        },
    )
    .await;
    insert_image(
        &pool,
        NewImage {
            sampler_name: Some("euler_ancestral".to_string()),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                sampler_name: Some("euler".to_string()),
                ..Default::default()
            }
        )
        .await,
        vec![euler],
        "exact match: euler_ancestral must not match euler"
    );
}

/// `jpg` matches both `.jpg` and `.jpeg`; the leading dot is optional.
#[sqlx::test(migrations = "./migrations")]
async fn file_type_jpg_matches_jpeg(pool: PgPool) {
    insert_image(
        &pool,
        NewImage {
            original_filename: "a.jpg".to_string(),
            ..Default::default()
        },
    )
    .await;
    insert_image(
        &pool,
        NewImage {
            original_filename: "b.jpeg".to_string(),
            ..Default::default()
        },
    )
    .await;
    let png = insert_image(&pool, NewImage::default()).await;

    assert_eq!(
        total(
            &pool,
            &SearchParams {
                file_type: Some(".jpg".to_string()),
                ..Default::default()
            }
        )
        .await,
        2
    );
    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                file_type: Some("png".to_string()),
                ..Default::default()
            }
        )
        .await,
        vec![png]
    );
}

// ---------------------------------------------------------------------------
// JSONB filters
// ---------------------------------------------------------------------------

/// Multiple tags are ANDed (every tag must be present), not ORed.
#[sqlx::test(migrations = "./migrations")]
async fn tags_are_anded(pool: PgPool) {
    let both = insert_image(
        &pool,
        NewImage {
            user_tags: json!(["landscape", "wallpaper"]),
            ..Default::default()
        },
    )
    .await;
    insert_image(
        &pool,
        NewImage {
            user_tags: json!(["landscape"]),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        total(
            &pool,
            &SearchParams {
                tags: vec!["landscape".to_string()],
                ..Default::default()
            }
        )
        .await,
        2
    );
    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                tags: vec!["landscape".to_string(), "wallpaper".to_string()],
                ..Default::default()
            }
        )
        .await,
        vec![both],
        "two tags mean both must be present"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn lora_name_matches_by_containment(pool: PgPool) {
    let with_lora = insert_image(
        &pool,
        NewImage {
            loras: json!([{"name": "detail_tweaker", "weight": 0.8}]),
            ..Default::default()
        },
    )
    .await;
    insert_image(
        &pool,
        NewImage {
            loras: json!([{"name": "other_lora", "weight": 1.0}]),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                lora_name: Some("detail_tweaker".to_string()),
                ..Default::default()
            }
        )
        .await,
        vec![with_lora],
        "containment ignores the other keys (weight) in the object"
    );
}

/// `is_xyz_grid=false` and `is_upscaled=false` must also match images whose
/// model_params lack the key entirely (NULL), not just those with a false value.
#[sqlx::test(migrations = "./migrations")]
async fn model_params_flag_filters_treat_missing_as_false(pool: PgPool) {
    let grid = insert_image(
        &pool,
        NewImage {
            model_params: json!({"is_xyz_grid": "true"}),
            ..Default::default()
        },
    )
    .await;
    let upscaled = insert_image(
        &pool,
        NewImage {
            model_params: json!({"hires_upscaler": "4x-UltraSharp"}),
            ..Default::default()
        },
    )
    .await;
    let plain = insert_image(&pool, NewImage::default()).await;

    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                is_xyz_grid: Some(true),
                ..Default::default()
            }
        )
        .await,
        vec![grid]
    );
    assert_eq!(
        total(
            &pool,
            &SearchParams {
                is_xyz_grid: Some(false),
                ..Default::default()
            }
        )
        .await,
        2,
        "missing key counts as not-a-grid"
    );
    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                is_upscaled: Some(true),
                ..Default::default()
            }
        )
        .await,
        vec![upscaled]
    );

    let not_upscaled = ids(
        &pool,
        &SearchParams {
            is_upscaled: Some(false),
            sort_by: "created_at".to_string(),
            ..Default::default()
        },
    )
    .await;
    assert!(not_upscaled.contains(&plain) && not_upscaled.contains(&grid));
    assert!(!not_upscaled.contains(&upscaled));
}

// ---------------------------------------------------------------------------
// full-text search (C1)
// ---------------------------------------------------------------------------

/// Insert three prompts used by the search tests.
async fn seed_prompts(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let girl = insert_image(
        &pool.clone(),
        NewImage {
            positive_prompt: Some("1girl, solo, (masterpiece:1.2), detailed".to_string()),
            ..Default::default()
        },
    )
    .await;
    let landscape = insert_image(
        pool,
        NewImage {
            positive_prompt: Some("landscape, mountain, sunset".to_string()),
            ..Default::default()
        },
    )
    .await;
    let japanese = insert_image(
        pool,
        NewImage {
            positive_prompt: Some("美少女イラスト, 夕焼け 空".to_string()),
            ..Default::default()
        },
    )
    .await;
    (girl, landscape, japanese)
}

fn search(q: &str) -> SearchParams {
    SearchParams {
        q: Some(q.to_string()),
        per_page: 100,
        ..Default::default()
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn search_matches_tokens_case_insensitively(pool: PgPool) {
    let (girl, _, _) = seed_prompts(&pool).await;

    assert_eq!(ids(&pool, &search("masterpiece")).await, vec![girl]);
    assert_eq!(
        ids(&pool, &search("MASTERPIECE")).await,
        vec![girl],
        "search is case-insensitive"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn search_terms_are_anded(pool: PgPool) {
    let (girl, _, _) = seed_prompts(&pool).await;

    assert_eq!(ids(&pool, &search("1girl solo")).await, vec![girl]);
    assert_eq!(
        total(&pool, &search("1girl mountain")).await,
        0,
        "terms from different images must not match either of them"
    );
}

/// The old implementation pasted the query into to_tsquery, so these characters
/// raised a syntax error that surfaced as an HTTP 500. websearch_to_tsquery
/// treats them as text.
#[sqlx::test(migrations = "./migrations")]
async fn search_never_errors_on_tsquery_metacharacters(pool: PgPool) {
    seed_prompts(&pool).await;

    for q in ["!", "|", "(", ")", "&", ":", "(masterpiece:1.2)", "a & | b"] {
        let result = image::list(&pool, &search(q)).await;
        assert!(
            result.is_ok(),
            "query {q:?} must not error, got {:?}",
            result.err()
        );
    }
}

/// `(masterpiece:1.2)` is how weights appear in prompts, so searching for the
/// literal text a user copied out of one has to work.
#[sqlx::test(migrations = "./migrations")]
async fn search_finds_weighted_tag_text(pool: PgPool) {
    let (girl, _, _) = seed_prompts(&pool).await;

    assert_eq!(ids(&pool, &search("(masterpiece:1.2)")).await, vec![girl]);
}

/// Japanese separated by spaces or commas tokenises normally.
#[sqlx::test(migrations = "./migrations")]
async fn search_matches_separated_japanese(pool: PgPool) {
    let (_, _, japanese) = seed_prompts(&pool).await;

    assert_eq!(ids(&pool, &search("夕焼け")).await, vec![japanese]);
}

/// Japanese without separators collapses into a single token, so this only
/// works through the trigram/ILIKE arm — the reason the query has two arms.
#[sqlx::test(migrations = "./migrations")]
async fn search_matches_japanese_substring(pool: PgPool) {
    let (_, _, japanese) = seed_prompts(&pool).await;

    assert_eq!(
        ids(&pool, &search("少女")).await,
        vec![japanese],
        "「少女」 must find 「美少女イラスト」"
    );
}

/// websearch_to_tsquery syntax that the old to_tsquery path could not express.
#[sqlx::test(migrations = "./migrations")]
async fn search_supports_websearch_operators(pool: PgPool) {
    let (girl, landscape, _) = seed_prompts(&pool).await;

    assert_eq!(
        ids(&pool, &search("\"mountain sunset\"")).await,
        vec![landscape],
        "quoted phrase matches adjacent tokens"
    );
    assert_eq!(
        total(&pool, &search("\"sunset mountain\"")).await,
        0,
        "phrase order matters"
    );

    let detailed_only = insert_image(
        &pool,
        NewImage {
            positive_prompt: Some("detailed, background".to_string()),
            ..Default::default()
        },
    )
    .await;
    let excluded = ids(&pool, &search("detailed -solo")).await;
    assert_eq!(
        excluded,
        vec![detailed_only],
        "-solo drops the image containing solo but keeps the other match"
    );
    assert!(!excluded.contains(&girl));
}

#[sqlx::test(migrations = "./migrations")]
async fn search_ignores_whitespace_only_queries(pool: PgPool) {
    seed_prompts(&pool).await;

    assert_eq!(
        total(&pool, &search("   ")).await,
        3,
        "a blank query is no filter at all"
    );
}

/// Images without a prompt must never match a text search (coalesce to '').
#[sqlx::test(migrations = "./migrations")]
async fn search_skips_images_without_prompt(pool: PgPool) {
    insert_image(&pool, NewImage::default()).await; // positive_prompt = NULL

    assert_eq!(total(&pool, &search("anything")).await, 0);
}

// ---------------------------------------------------------------------------
// dimensions, dates, seed
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn orientation_and_dimension_filters(pool: PgPool) {
    let portrait = insert_image(
        &pool,
        NewImage {
            width: 832,
            height: 1216,
            ..Default::default()
        },
    )
    .await;
    let landscape = insert_image(
        &pool,
        NewImage {
            width: 1216,
            height: 832,
            ..Default::default()
        },
    )
    .await;
    let square = insert_image(&pool, NewImage::default()).await;

    for (o, expected) in [
        ("portrait", portrait),
        ("landscape", landscape),
        ("square", square),
    ] {
        assert_eq!(
            ids(
                &pool,
                &SearchParams {
                    orientation: Some(o.to_string()),
                    ..Default::default()
                }
            )
            .await,
            vec![expected],
            "orientation={o}"
        );
    }

    assert_eq!(
        total(
            &pool,
            &SearchParams {
                orientation: Some("diagonal".to_string()),
                ..Default::default()
            }
        )
        .await,
        3,
        "an unknown orientation adds no condition (silently ignored)"
    );

    assert_eq!(
        total(
            &pool,
            &SearchParams {
                min_width: Some(1000),
                ..Default::default()
            }
        )
        .await,
        2
    );
    assert_eq!(
        total(
            &pool,
            &SearchParams {
                min_height: Some(1000),
                ..Default::default()
            }
        )
        .await,
        2
    );
}

/// date_from accepts RFC3339, naive datetime and date-only; anything else is
/// ignored rather than rejected.
#[sqlx::test(migrations = "./migrations")]
async fn date_from_accepted_formats(pool: PgPool) {
    insert_image(
        &pool,
        NewImage {
            created_at: utc(2026, 1, 1, 0, 0, 0),
            ..Default::default()
        },
    )
    .await;
    insert_image(
        &pool,
        NewImage {
            created_at: utc(2026, 6, 1, 0, 0, 0),
            ..Default::default()
        },
    )
    .await;

    for value in [
        "2026-03-01T00:00:00Z",
        "2026-03-01T00:00:00",
        "2026-03-01",
    ] {
        assert_eq!(
            total(
                &pool,
                &SearchParams {
                    date_from: Some(value.to_string()),
                    ..Default::default()
                }
            )
            .await,
            1,
            "date_from={value}"
        );
    }

    assert_eq!(
        total(
            &pool,
            &SearchParams {
                date_from: Some("not-a-date".to_string()),
                ..Default::default()
            }
        )
        .await,
        2,
        "an unparseable date_from is ignored, not an error"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn seed_exact_and_tolerance(pool: PgPool) {
    let exact = insert_image(
        &pool,
        NewImage {
            seed: Some(1_000_000),
            ..Default::default()
        },
    )
    .await;
    let near = insert_image(
        &pool,
        NewImage {
            seed: Some(1_000_200),
            ..Default::default()
        },
    )
    .await;
    insert_image(
        &pool,
        NewImage {
            seed: Some(9_999_999),
            ..Default::default()
        },
    )
    .await;
    insert_image(&pool, NewImage::default()).await; // seed IS NULL

    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                seed: Some(1_000_000),
                ..Default::default()
            }
        )
        .await,
        vec![exact]
    );

    let within = ids(
        &pool,
        &SearchParams {
            seed: Some(1_000_000),
            seed_tolerance: Some(300),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(within.len(), 2);
    assert!(within.contains(&exact) && within.contains(&near));

    assert_eq!(
        total(
            &pool,
            &SearchParams {
                seed: Some(1_000_000),
                seed_tolerance: Some(0),
                ..Default::default()
            }
        )
        .await,
        1,
        "tolerance=0 falls back to exact match"
    );
}

/// A tolerance window around an extreme seed must not overflow i64.
#[sqlx::test(migrations = "./migrations")]
async fn seed_tolerance_saturates_at_i64_bounds(pool: PgPool) {
    let max = insert_image(
        &pool,
        NewImage {
            seed: Some(i64::MAX),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                seed: Some(i64::MAX),
                seed_tolerance: Some(300),
                ..Default::default()
            }
        )
        .await,
        vec![max]
    );
}

// ---------------------------------------------------------------------------
// sorting and pagination
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn sort_column_whitelist_and_fallback(pool: PgPool) {
    let small = insert_image(
        &pool,
        NewImage {
            width: 512,
            rating: 5,
            created_at: utc(2026, 1, 1, 0, 0, 0),
            ..Default::default()
        },
    )
    .await;
    let large = insert_image(
        &pool,
        NewImage {
            width: 2048,
            rating: 1,
            created_at: utc(2026, 1, 1, 0, 1, 0),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                sort_by: "width".to_string(),
                sort_order: "asc".to_string(),
                ..Default::default()
            }
        )
        .await,
        vec![small, large]
    );
    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                sort_by: "rating".to_string(),
                sort_order: "desc".to_string(),
                ..Default::default()
            }
        )
        .await,
        vec![small, large]
    );
    assert_eq!(
        ids(
            &pool,
            &SearchParams {
                sort_by: "; DROP TABLE images".to_string(),
                ..Default::default()
            }
        )
        .await,
        vec![large, small],
        "an unknown sort column silently falls back to created_at DESC"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn pagination_slices_the_result_set(pool: PgPool) {
    let seq = insert_sequence(&pool, 5).await; // oldest .. newest

    let page1 = ids(
        &pool,
        &SearchParams {
            per_page: 2,
            page: 1,
            sort_order: "asc".to_string(),
            ..Default::default()
        },
    )
    .await;
    let page3 = ids(
        &pool,
        &SearchParams {
            per_page: 2,
            page: 3,
            sort_order: "asc".to_string(),
            ..Default::default()
        },
    )
    .await;

    assert_eq!(page1, vec![seq[0], seq[1]]);
    assert_eq!(page3, vec![seq[4]], "last page holds the remainder");
    assert_eq!(
        total(
            &pool,
            &SearchParams {
                per_page: 2,
                page: 1,
                ..Default::default()
            }
        )
        .await,
        5,
        "total ignores pagination"
    );
}

/// Filters must apply identically to the count query and the page query,
/// otherwise the UI shows a total that does not match the rows.
#[sqlx::test(migrations = "./migrations")]
async fn count_and_page_queries_agree(pool: PgPool) {
    for i in 0..7i16 {
        insert_image(
            &pool,
            NewImage {
                rating: i % 6,
                is_favorite: i % 2 == 0,
                original_filename: format!("i{i}.png"),
                ..Default::default()
            },
        )
        .await;
    }

    let p = SearchParams {
        min_rating: Some(2),
        is_favorite: Some(true),
        per_page: 100,
        ..Default::default()
    };
    let result = image::list(&pool, &p).await.expect("list");
    assert_eq!(result.total as usize, result.items.len());
}
