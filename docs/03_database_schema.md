# データベーススキーマ設計

## 設計方針

- **ハイブリッド方式**: 高頻度検索項目はカラム、可変・拡張データはJSONB
- **UUID v7**: 時系列ソート可能なIDを採用
- **論理削除対応**: `deleted_at`カラムで二段階削除を実現

---

## テーブル定義

### images テーブル

```sql
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE images (
    -- ========== 主キー ==========
    id UUID PRIMARY KEY,  -- UUID v7（アプリ側で生成）
    
    -- ========== ソース情報 ==========
    source_tool VARCHAR(20) NOT NULL,
    -- 'comfyui', 'a1111', 'forge', 'novelai', 'unknown'
    
    model_type VARCHAR(20),
    -- 'sd15', 'sdxl', 'pony', 'illustrious', 'flux', 'qwen', 'other'
    
    has_metadata BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- ========== ファイル情報 ==========
    original_filename VARCHAR(255) NOT NULL,
    storage_path VARCHAR(512) NOT NULL,
    thumbnail_path VARCHAR(512) NOT NULL,
    file_hash VARCHAR(64) NOT NULL UNIQUE,  -- SHA-256
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    
    -- ========== 共通検索カラム ==========
    positive_prompt TEXT,
    negative_prompt TEXT,
    model_name VARCHAR(255),
    sampler_name VARCHAR(50),
    scheduler VARCHAR(50),
    steps INTEGER,
    cfg_scale NUMERIC(5,2),
    seed BIGINT,
    
    -- ========== 拡張データ（JSONB） ==========
    loras JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- 例: [{"name": "lora1", "weight": 0.8, "hash": "abc123"}, ...]
    
    controlnets JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- 例: [{"model": "openpose", "weight": 1.0, "guidance_start": 0.0, "guidance_end": 1.0}, ...]
    
    embeddings JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- 例: [{"name": "embedding1", "hash": "def456"}, ...]
    
    model_params JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- モデル固有パラメータ（Flux: guidance, Qwen: true_cfg_scale 等）
    
    workflow_extras JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- その他ワークフロー情報
    
    -- ========== 生メタデータ ==========
    raw_metadata JSONB,
    -- パース前の完全なメタデータ（デバッグ・再パース用）
    
    -- ========== ユーザーデータ ==========
    rating SMALLINT NOT NULL DEFAULT 0,
    -- 0〜5（0は未評価）
    CONSTRAINT rating_range CHECK (rating >= 0 AND rating <= 5),
    
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    
    needs_improvement BOOLEAN NOT NULL DEFAULT FALSE,
    -- 改善対象フラグ
    
    user_tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- 例: ["portrait", "landscape", "wip"]
    
    user_memo TEXT,
    -- ユーザーメモ
    
    -- ========== タイムスタンプ ==========
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,  -- 論理削除用（NULLなら未削除）
    
    -- ========== 制約 ==========
    CONSTRAINT valid_dimensions CHECK (width > 0 AND height > 0),
    CONSTRAINT valid_file_size CHECK (file_size_bytes > 0)
);

-- テーブルコメント
COMMENT ON TABLE images IS 'AI生成画像のメタデータと管理情報';
COMMENT ON COLUMN images.id IS 'UUID v7（時系列ソート可能）';
COMMENT ON COLUMN images.source_tool IS '生成ツール（comfyui/a1111/forge/novelai/unknown）';
COMMENT ON COLUMN images.model_type IS 'モデルタイプ（sd15/sdxl/pony/illustrious/flux/qwen/other）';
COMMENT ON COLUMN images.file_hash IS 'SHA-256ハッシュ（重複チェック用）';
COMMENT ON COLUMN images.model_params IS 'モデル固有パラメータ（JSONB）';
COMMENT ON COLUMN images.deleted_at IS '論理削除日時（NULLは未削除）';
```

---

## インデックス定義

```sql
-- ========== 基本カラムインデックス ==========
CREATE INDEX idx_images_source_tool ON images(source_tool);
CREATE INDEX idx_images_model_type ON images(model_type);
CREATE INDEX idx_images_model_name ON images(model_name);
CREATE INDEX idx_images_sampler_name ON images(sampler_name);
CREATE INDEX idx_images_steps ON images(steps);
CREATE INDEX idx_images_cfg_scale ON images(cfg_scale);
CREATE INDEX idx_images_rating ON images(rating);
CREATE INDEX idx_images_created_at ON images(created_at DESC);

-- ========== フラグインデックス（部分インデックス） ==========
CREATE INDEX idx_images_is_favorite ON images(is_favorite) 
    WHERE is_favorite = TRUE;
CREATE INDEX idx_images_needs_improvement ON images(needs_improvement) 
    WHERE needs_improvement = TRUE;
CREATE INDEX idx_images_deleted ON images(deleted_at) 
    WHERE deleted_at IS NOT NULL;

-- ========== 部分一致検索用（トライグラム） ==========
CREATE INDEX idx_images_model_name_trgm ON images 
    USING gin(model_name gin_trgm_ops);
CREATE INDEX idx_images_original_filename_trgm ON images 
    USING gin(original_filename gin_trgm_ops);

-- ========== 全文検索用 ==========
CREATE INDEX idx_images_positive_prompt_fts ON images 
    USING gin(to_tsvector('english', COALESCE(positive_prompt, '')));
CREATE INDEX idx_images_negative_prompt_fts ON images 
    USING gin(to_tsvector('english', COALESCE(negative_prompt, '')));

-- ========== JSONB用GINインデックス ==========
CREATE INDEX idx_images_loras ON images USING gin(loras jsonb_path_ops);
CREATE INDEX idx_images_controlnets ON images USING gin(controlnets jsonb_path_ops);
CREATE INDEX idx_images_user_tags ON images USING gin(user_tags jsonb_path_ops);
CREATE INDEX idx_images_model_params ON images USING gin(model_params jsonb_path_ops);

-- ========== 複合インデックス ==========
CREATE INDEX idx_images_list_default ON images(created_at DESC) 
    WHERE deleted_at IS NULL;
CREATE INDEX idx_images_model_rating ON images(model_type, rating DESC) 
    WHERE deleted_at IS NULL;

-- ========== ユニーク制約 ==========
-- file_hashは既にUNIQUE制約あり
```

---

## JSONB フィールド詳細

### loras

```json
[
  {
    "name": "animeMix_v1",
    "weight": 0.8,
    "weight_clip": 0.8,
    "hash": "abc123def456"
  }
]
```

| フィールド | 型 | 説明 |
|-----------|-----|------|
| name | string | LoRA名 |
| weight | number | モデル適用強度 |
| weight_clip | number | CLIP適用強度（オプション） |
| hash | string | ファイルハッシュ（オプション） |

### controlnets

```json
[
  {
    "model": "control_v11p_sd15_openpose",
    "weight": 1.0,
    "guidance_start": 0.0,
    "guidance_end": 1.0,
    "preprocessor": "openpose"
  }
]
```

| フィールド | 型 | 説明 |
|-----------|-----|------|
| model | string | ControlNetモデル名 |
| weight | number | 適用強度 |
| guidance_start | number | 適用開始タイミング |
| guidance_end | number | 適用終了タイミング |
| preprocessor | string | 前処理方式（オプション） |

### model_params（モデル別）

**Flux.1:**
```json
{
  "guidance": 3.5,
  "shift": 1.0,
  "t5_encoder": "t5xxl_fp16"
}
```

**Qwen-Image:**
```json
{
  "true_cfg_scale": 4.0,
  "lightning_lora": true,
  "lightning_steps": 8
}
```

**SDXL/Pony/Illustrious:**
```json
{
  "clip_skip": 2,
  "vae": "sdxl_vae.safetensors",
  "refiner_model": null,
  "refiner_switch_at": null
}
```

### user_tags

```json
["portrait", "high_quality", "character_design"]
```

単純な文字列配列。

### workflow_extras

```json
{
  "comfyui_version": "0.2.0",
  "custom_nodes": ["ComfyUI-Impact-Pack", "ComfyUI-ControlNet-Aux"],
  "node_count": 15,
  "execution_time_ms": 12500
}
```

ワークフローに関する追加情報。

---

## 検索クエリ例

### 基本検索（カラム）

```sql
-- モデルタイプとサンプラーで絞り込み
SELECT * FROM images
WHERE model_type = 'pony'
  AND sampler_name = 'euler_ancestral'
  AND deleted_at IS NULL
ORDER BY created_at DESC
LIMIT 24 OFFSET 0;
```

### プロンプト全文検索

```sql
-- 複数キーワードAND検索
SELECT * FROM images
WHERE to_tsvector('english', COALESCE(positive_prompt, '')) 
      @@ to_tsquery('english', '1girl & masterpiece')
  AND deleted_at IS NULL
ORDER BY created_at DESC;
```

### JSONB検索（LoRA）

```sql
-- 特定のLoRAを使用している画像
SELECT * FROM images
WHERE loras @> '[{"name": "animeMix_v1"}]'::jsonb
  AND deleted_at IS NULL;

-- LoRA名の部分一致
SELECT * FROM images
WHERE EXISTS (
    SELECT 1 FROM jsonb_array_elements(loras) AS lora
    WHERE lora->>'name' ILIKE '%anime%'
)
  AND deleted_at IS NULL;
```

### JSONB検索（ユーザータグ）

```sql
-- 特定タグを持つ画像
SELECT * FROM images
WHERE user_tags ? 'portrait'
  AND deleted_at IS NULL;

-- 複数タグAND
SELECT * FROM images
WHERE user_tags ?& ARRAY['portrait', 'high_quality']
  AND deleted_at IS NULL;
```

### モデル固有パラメータ検索

```sql
-- Flux.1でguidance 3.0以上
SELECT * FROM images
WHERE model_type = 'flux'
  AND (model_params->>'guidance')::numeric >= 3.0
  AND deleted_at IS NULL;
```

### 削除済み画像の取得（ゴミ箱）

```sql
SELECT * FROM images
WHERE deleted_at IS NOT NULL
ORDER BY deleted_at DESC;
```

---

## マイグレーション方針

- Alembicを使用
- 初期マイグレーションでテーブル・インデックスを一括作成
- 将来の変更は個別マイグレーションファイルで管理

---

## 初期化SQL

`db/init/01_init.sql` として配置し、docker-compose起動時に自動実行。

```sql
-- 拡張機能
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- テーブル作成（上記のCREATE TABLE文）

-- インデックス作成（上記のCREATE INDEX文）

-- updated_at自動更新トリガー
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_images_updated_at
    BEFORE UPDATE ON images
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```
