# API設計

## 概要

- ベースURL: `/api`
- 認証: セッションベース（Cookie）
- レスポンス形式: JSON
- エラー形式: RFC 7807 Problem Details

---

## 認証API

### POST /api/auth/login

ログイン処理。

**リクエスト:**
```json
{
  "username": "admin",
  "password": "password123"
}
```

**レスポンス（成功）:** `200 OK`
```json
{
  "message": "Login successful",
  "user": {
    "username": "admin"
  }
}
```

**レスポンス（失敗）:** `401 Unauthorized`
```json
{
  "type": "authentication_error",
  "title": "Authentication Failed",
  "status": 401,
  "detail": "Invalid username or password"
}
```

### POST /api/auth/logout

ログアウト処理。

**レスポンス:** `200 OK`
```json
{
  "message": "Logout successful"
}
```

### GET /api/auth/me

現在のログインユーザー情報取得。

**レスポンス（認証済み）:** `200 OK`
```json
{
  "username": "admin",
  "authenticated": true
}
```

**レスポンス（未認証）:** `401 Unauthorized`

---

## 画像API

### GET /api/images

画像一覧取得（検索・ページネーション対応）。

**クエリパラメータ:**

| パラメータ | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| page | integer | No | ページ番号（デフォルト: 1） |
| per_page | integer | No | 1ページあたり件数（12/24/48/96、デフォルト: 24） |
| sort | string | No | ソート項目（created_at/rating/file_size、デフォルト: created_at） |
| order | string | No | ソート順（asc/desc、デフォルト: desc） |
| q | string | No | プロンプト全文検索キーワード |
| model_type | string | No | モデルタイプ（sd15/sdxl/pony/illustrious/flux/qwen/other） |
| model_name | string | No | モデル名（部分一致） |
| sampler | string | No | サンプラー名（完全一致） |
| lora_name | string | No | LoRA名（完全一致） |
| source_tool | string | No | ソースツール（comfyui/a1111/forge/novelai/unknown） |
| min_steps | integer | No | 最小ステップ数 |
| max_steps | integer | No | 最大ステップ数 |
| min_cfg | number | No | 最小CFG |
| max_cfg | number | No | 最大CFG |
| min_rating | integer | No | 最小評価 |
| max_rating | integer | No | 最大評価 |
| is_favorite | boolean | No | お気に入りフィルタ |
| needs_improvement | boolean | No | 改善対象フィルタ |
| tags | string | No | タグ（カンマ区切りでAND検索） |
| include_deleted | boolean | No | 削除済みを含めるか（デフォルト: false） |

**レスポンス:** `200 OK`
```json
{
  "items": [
    {
      "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
      "thumbnail_url": "/storage/01/93/01936f4e-5b3a-7000-8000-1234abcd5678_thumb.webp",
      "width": 1024,
      "height": 1024,
      "model_type": "pony",
      "model_name": "ponyDiffusionV6XL",
      "rating": 4,
      "is_favorite": false,
      "needs_improvement": false,
      "user_tags": ["portrait", "anime"],
      "created_at": "2025-01-15T10:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 24,
    "total_items": 1250,
    "total_pages": 53,
    "has_next": true,
    "has_prev": false
  }
}
```

### GET /api/images/{id}

画像詳細取得（前後画像のナビゲーション情報付き）。

**クエリパラメータ:**

一覧取得と同じ検索パラメータを受け取り、その検索条件内での前後画像IDを返す。

| パラメータ | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| q | string | No | プロンプト全文検索キーワード |
| model_type | string | No | モデルタイプ |
| source_tool | string | No | ソースツール |
| min_rating | integer | No | 最小評価 |
| exact_rating | integer | No | 評価完全一致 |
| is_favorite | boolean | No | お気に入りフィルタ |
| is_xyz_grid | boolean | No | XYZ Gridフィルタ |
| is_upscaled | boolean | No | アップスケール済みフィルタ |
| sort_by | string | No | ソート項目（デフォルト: created_at） |
| sort_order | string | No | ソート順（デフォルト: desc） |
| ... | | | その他の検索パラメータも同様 |

**レスポンス:** `200 OK`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "original_url": "/storage/01/93/01936f4e-5b3a-7000-8000-1234abcd5678.png",
  "thumbnail_url": "/storage/01/93/01936f4e-5b3a-7000-8000-1234abcd5678_thumb.webp",
  "original_filename": "ComfyUI_00123_.png",
  "width": 1024,
  "height": 1024,
  "file_size_bytes": 2458624,
  "source_tool": "comfyui",
  "model_type": "pony",
  "has_metadata": true,
  "positive_prompt": "1girl, beautiful, masterpiece, best quality",
  "negative_prompt": "ugly, blurry, low quality",
  "model_name": "ponyDiffusionV6XL",
  "sampler_name": "euler_ancestral",
  "scheduler": "normal",
  "steps": 25,
  "cfg_scale": 7.0,
  "seed": 1234567890,
  "loras": [
    {"name": "animeMix_v1", "weight": 0.8}
  ],
  "controlnets": [],
  "model_params": {
    "clip_skip": 2
  },
  "raw_metadata": { },
  "rating": 4,
  "is_favorite": false,
  "needs_improvement": false,
  "user_tags": ["portrait", "anime"],
  "user_memo": null,
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z",
  "deleted_at": null,
  "prev_id": "01936f4e-4a29-7000-8000-0987dcba4321",
  "next_id": "01936f4e-6c4b-7000-8000-5678efgh9012"
}
```

**レスポンス（404）:** `404 Not Found`
```json
{
  "type": "not_found",
  "title": "Image Not Found",
  "status": 404,
  "detail": "Image with id '01936f4e-5b3a-7000-8000-1234abcd5678' not found"
}
```

### PATCH /api/images/{id}

画像情報更新（評価、タグ、メモ等）。

**リクエスト:**
```json
{
  "rating": 5,
  "is_favorite": true,
  "needs_improvement": false,
  "user_tags": ["portrait", "anime", "high_quality"],
  "user_memo": "お気に入りの一枚"
}
```

※ 含まれるフィールドのみ更新

**レスポンス:** `200 OK`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "rating": 5,
  "is_favorite": true,
  "needs_improvement": false,
  "user_tags": ["portrait", "anime", "high_quality"],
  "user_memo": "お気に入りの一枚",
  "updated_at": "2025-01-15T11:00:00Z"
}
```

### DELETE /api/images/{id}

画像の論理削除。

**レスポンス:** `200 OK`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "deleted_at": "2025-01-15T12:00:00Z",
  "message": "Image moved to trash"
}
```

### POST /api/images/{id}/restore

論理削除した画像の復元。

**レスポンス:** `200 OK`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "message": "Image restored"
}
```

### DELETE /api/images/{id}/permanent

画像の物理削除（完全削除）。

**レスポンス:** `200 OK`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "message": "Image permanently deleted"
}
```

---

## 一括操作API

### POST /api/bulk/update

複数画像の一括更新。

**リクエスト:**
```json
{
  "ids": [
    "01936f4e-5b3a-7000-8000-1234abcd5678",
    "01936f4e-6c4b-7000-8000-5678efgh9012"
  ],
  "rating": 4,
  "is_favorite": true,
  "needs_improvement": false,
  "add_tags": ["selected"],
  "remove_tags": ["wip"]
}
```

※ `ids` 以外のフィールドはすべてオプション。指定したフィールドのみ更新される。

**レスポンス:** `200 OK`
```json
{
  "message": "2 images updated"
}
```

### POST /api/bulk/delete

複数画像の一括削除（論理削除または物理削除）。

**リクエスト:**
```json
{
  "ids": [
    "01936f4e-5b3a-7000-8000-1234abcd5678",
    "01936f4e-6c4b-7000-8000-5678efgh9012"
  ],
  "permanent": false
}
```

| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| ids | string[] | Yes | 削除対象の画像ID |
| permanent | boolean | No | true: 物理削除、false: 論理削除（デフォルト: false） |

**レスポンス:** `200 OK`
```json
{
  "message": "2 images deleted"
}
```

### POST /api/bulk/restore

論理削除した複数画像の一括復元。

**リクエスト:**
```json
{
  "ids": [
    "01936f4e-5b3a-7000-8000-1234abcd5678",
    "01936f4e-6c4b-7000-8000-5678efgh9012"
  ]
}
```

**レスポンス:** `200 OK`
```json
{
  "message": "2 images restored"
}

---

## タグAPI

### GET /api/tags

使用中のタグ一覧取得。

**レスポンス:** `200 OK`
```json
{
  "tags": [
    {"name": "portrait", "count": 150},
    {"name": "landscape", "count": 80},
    {"name": "anime", "count": 320},
    {"name": "wip", "count": 25}
  ]
}
```

### GET /api/tags/suggest

タグのサジェスト（入力補完用）。

**クエリパラメータ:**
| パラメータ | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| q | string | Yes | 検索文字列 |
| limit | integer | No | 最大件数（デフォルト: 10） |

**レスポンス:** `200 OK`
```json
{
  "suggestions": ["portrait", "pose", "poster"]
}
```

---

## エクスポートAPI

### POST /api/export

選択画像のZIPエクスポート。

**リクエスト:**
```json
{
  "ids": [
    "01936f4e-5b3a-7000-8000-1234abcd5678",
    "01936f4e-6c4b-7000-8000-5678efgh9012"
  ]
}
```

**レスポンス:** `200 OK`
- Content-Type: `application/zip`
- Content-Disposition: `attachment; filename="export_20250115_120000.zip"`
- Body: ZIPファイルバイナリ

**レスポンス（件数超過）:** `400 Bad Request`
```json
{
  "type": "validation_error",
  "title": "Export Limit Exceeded",
  "status": 400,
  "detail": "Maximum 100 images can be exported at once. Requested: 150"
}
```

---

## メタ情報API

### GET /api/meta/filters

フィルタ選択肢取得（ドロップダウン用）。

**レスポンス:** `200 OK`
```json
{
  "model_types": ["sd15", "sdxl", "pony", "illustrious", "flux", "qwen", "other"],
  "source_tools": ["comfyui", "a1111", "forge", "novelai", "unknown"],
  "samplers": ["euler", "euler_ancestral", "dpm_2", "dpm_2_ancestral", "heun", "dpm_fast", "dpm_adaptive", "lms", "dpmpp_2s_ancestral", "dpmpp_sde", "dpmpp_2m", "dpmpp_2m_sde"],
  "schedulers": ["normal", "karras", "exponential", "sgm_uniform", "simple", "ddim_uniform"]
}
```

### GET /api/meta/stats

統計情報取得。

**レスポンス:** `200 OK`
```json
{
  "total_images": 1250,
  "total_size_bytes": 3145728000,
  "by_model_type": {
    "pony": 450,
    "illustrious": 320,
    "flux": 280,
    "sdxl": 150,
    "qwen": 50
  },
  "by_rating": {
    "0": 200,
    "1": 50,
    "2": 100,
    "3": 300,
    "4": 400,
    "5": 200
  },
  "favorites_count": 85,
  "needs_improvement_count": 45,
  "deleted_count": 30
}
```

### GET /api/stats/models-for-analysis

分析用モデル一覧取得（画像数が指定数以上のモデルのみ）。

**クエリパラメータ:**
- `min_count` (optional, default: 5): 最小画像数

**レスポンス:** `200 OK`
```json
{
  "models": ["animagine-xl-3.1", "pony-diffusion-v6", "illustrious-xl-v0.1"]
}
```

### GET /api/stats/loras-for-filter

検索フィルター用LoRA一覧取得。

**クエリパラメータ:**
- `min_count` (optional, default: 1): 最小使用回数

**レスポンス:** `200 OK`
```json
{
  "loras": ["detail_tweaker_xl", "add_more_details", "brightness_slider"]
}
```

---

## 検索プリセットAPI

### GET /api/search-presets

検索プリセット一覧取得（作成日時降順）。

**レスポンス:** `200 OK`
```json
[
  {
    "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
    "name": "高評価SDXL",
    "filters": {
      "model_type": "sdxl",
      "min_rating": 4
    },
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T10:30:00Z"
  }
]
```

### POST /api/search-presets

検索プリセット新規作成。

**リクエスト:**
```json
{
  "name": "高評価SDXL",
  "filters": {
    "model_type": "sdxl",
    "min_rating": 4,
    "sort_by": "rating",
    "sort_order": "desc"
  }
}
```

**レスポンス:** `201 Created`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "name": "高評価SDXL",
  "filters": {
    "model_type": "sdxl",
    "min_rating": 4,
    "sort_by": "rating",
    "sort_order": "desc"
  },
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z"
}
```

### PUT /api/search-presets/{id}

検索プリセット更新。

**リクエスト:**
```json
{
  "name": "高評価SDXL（更新）",
  "filters": {
    "model_type": "sdxl",
    "min_rating": 5
  }
}
```

**レスポンス:** `200 OK`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "name": "高評価SDXL（更新）",
  "filters": {
    "model_type": "sdxl",
    "min_rating": 5
  },
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T11:00:00Z"
}
```

### DELETE /api/search-presets/{id}

検索プリセット削除。

**レスポンス:** `200 OK`
```json
{
  "message": "Search preset deleted successfully"
}
```

---

## スマートフォルダAPI

### GET /api/smart-folders

スマートフォルダ一覧取得（作成日時降順）。

**レスポンス:** `200 OK`
```json
[
  {
    "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
    "name": "お気に入り",
    "icon": "heart.fill",
    "filters": {
      "is_favorite": true
    },
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T10:30:00Z"
  }
]
```

### POST /api/smart-folders

スマートフォルダ新規作成。

**リクエスト:**
```json
{
  "name": "高評価作品",
  "icon": "star.fill",
  "filters": {
    "min_rating": 4,
    "sort_by": "rating",
    "sort_order": "desc"
  }
}
```

**レスポンス:** `201 Created`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "name": "高評価作品",
  "icon": "star.fill",
  "filters": {
    "min_rating": 4,
    "sort_by": "rating",
    "sort_order": "desc"
  },
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z"
}
```

### GET /api/smart-folders/{id}

スマートフォルダ詳細取得。

**レスポンス:** `200 OK`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "name": "高評価作品",
  "icon": "star.fill",
  "filters": {
    "min_rating": 4
  },
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z"
}
```

### PUT /api/smart-folders/{id}

スマートフォルダ更新。

**リクエスト:**
```json
{
  "name": "高評価作品（更新）",
  "icon": "star.circle.fill",
  "filters": {
    "min_rating": 5
  }
}
```

※ 含まれるフィールドのみ更新

**レスポンス:** `200 OK`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "name": "高評価作品（更新）",
  "icon": "star.circle.fill",
  "filters": {
    "min_rating": 5
  },
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T11:00:00Z"
}
```

### DELETE /api/smart-folders/{id}

スマートフォルダ削除。

**レスポンス:** `200 OK`
```json
{
  "message": "Smart folder deleted successfully"
}
```

---

## Showcase API

### GET /api/showcases

Showcase一覧取得（作成日時降順）。

**レスポンス:** `200 OK`
```json
[
  {
    "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
    "name": "ベスト作品集",
    "description": "お気に入りの作品をまとめたShowcase",
    "icon": null,
    "cover_image_id": "01936f4e-6c4b-7000-8000-5678efgh9012",
    "cover_thumbnail_path": "2024/01/15/01936f4e-6c4b-7000-8000-5678efgh9012_thumb.webp",
    "image_count": 25,
    "created_at": "2025-01-15T10:30:00Z",
    "updated_at": "2025-01-15T10:30:00Z"
  }
]
```

### POST /api/showcases

Showcase新規作成。

**リクエスト:**
```json
{
  "name": "ベスト作品集",
  "description": "お気に入りの作品をまとめたShowcase",
  "icon": null
}
```

**レスポンス:** `201 Created`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "name": "ベスト作品集",
  "description": "お気に入りの作品をまとめたShowcase",
  "icon": null,
  "cover_image_id": null,
  "cover_thumbnail_path": null,
  "image_count": 0,
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z"
}
```

### POST /api/showcases/check-images

指定した画像がどのShowcaseに含まれているかをチェック。

**リクエスト:**
```json
{
  "image_ids": [
    "01936f4e-5b3a-7000-8000-1234abcd5678",
    "01936f4e-6c4b-7000-8000-5678efgh9012"
  ]
}
```

**レスポンス:** `200 OK`
```json
[
  {
    "showcase_id": "01936f4e-7d5c-7000-8000-9012ijkl3456",
    "existing_count": 2
  },
  {
    "showcase_id": "01936f4e-8e6d-7000-8000-3456mnop7890",
    "existing_count": 1
  }
]
```

### GET /api/showcases/{id}

Showcase詳細取得（画像一覧含む）。

**レスポンス:** `200 OK`
```json
{
  "id": "01936f4e-5b3a-7000-8000-1234abcd5678",
  "name": "ベスト作品集",
  "description": "お気に入りの作品をまとめたShowcase",
  "icon": null,
  "cover_image_id": "01936f4e-6c4b-7000-8000-5678efgh9012",
  "cover_thumbnail_path": "2024/01/15/01936f4e-6c4b-7000-8000-5678efgh9012_thumb.webp",
  "image_count": 25,
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:30:00Z",
  "images": [
    {
      "id": "01936f4e-6c4b-7000-8000-5678efgh9012",
      "storage_path": "2024/01/15/01936f4e-6c4b-7000-8000-5678efgh9012.png",
      "thumbnail_path": "2024/01/15/01936f4e-6c4b-7000-8000-5678efgh9012_thumb.webp",
      "sort_order": 0,
      "added_at": "2025-01-15T10:35:00Z"
    }
  ]
}
```

### PUT /api/showcases/{id}

Showcase更新。

**リクエスト:**
```json
{
  "name": "ベスト作品集（更新）",
  "description": "更新した説明文",
  "cover_image_id": "01936f4e-6c4b-7000-8000-5678efgh9012"
}
```

※ 含まれるフィールドのみ更新

**レスポンス:** `200 OK`

### DELETE /api/showcases/{id}

Showcase削除。

**レスポンス:** `200 OK`
```json
{
  "message": "Showcase deleted successfully"
}
```

### POST /api/showcases/{id}/images

Showcaseに画像を追加。

**リクエスト:**
```json
{
  "image_ids": [
    "01936f4e-5b3a-7000-8000-1234abcd5678",
    "01936f4e-6c4b-7000-8000-5678efgh9012"
  ]
}
```

**レスポンス:** `200 OK`
```json
{
  "message": "Added 2 images to showcase"
}
```

※ 既に追加済みの画像はスキップされる

### DELETE /api/showcases/{id}/images

Showcaseから画像を削除。

**リクエスト:**
```json
{
  "image_ids": [
    "01936f4e-5b3a-7000-8000-1234abcd5678"
  ]
}
```

**レスポンス:** `200 OK`
```json
{
  "message": "Removed 1 images from showcase"
}
```

### PUT /api/showcases/{id}/images/reorder

Showcase内の画像の並び順を変更。

**リクエスト:**
```json
{
  "image_ids": [
    "01936f4e-6c4b-7000-8000-5678efgh9012",
    "01936f4e-5b3a-7000-8000-1234abcd5678"
  ]
}
```

※ 新しい順序で画像IDを指定

**レスポンス:** `200 OK`
```json
{
  "message": "Images reordered successfully"
}
```

---

## エラーレスポンス形式

RFC 7807 Problem Details準拠。

```json
{
  "type": "error_type",
  "title": "Human Readable Title",
  "status": 400,
  "detail": "Detailed error message",
  "instance": "/api/images/invalid-id"
}
```

### HTTPステータスコード

| コード | 用途 |
|--------|------|
| 200 | 成功 |
| 201 | 作成成功 |
| 400 | バリデーションエラー |
| 401 | 認証エラー |
| 403 | 権限エラー |
| 404 | リソース不存在 |
| 409 | 競合（重複等） |
| 500 | サーバーエラー |
