export interface ShowcaseImageInfo {
  id: string
  storage_path: string
  thumbnail_path: string
  sort_order: number
  added_at: string
}

export interface Showcase {
  id: string
  name: string
  description: string | null
  icon: string | null
  cover_image_id: string | null
  cover_thumbnail_path: string | null
  image_count: number
  created_at: string
  updated_at: string
}

export interface ShowcaseDetail extends Showcase {
  images: ShowcaseImageInfo[]
}

export interface ShowcaseCreate {
  name: string
  description?: string | null
  icon?: string | null
}

export interface ShowcaseUpdate {
  name?: string
  description?: string | null
  icon?: string | null
  cover_image_id?: string | null
}

export interface ShowcaseImageAdd {
  image_ids: string[]
}

export interface ShowcaseImageRemove {
  image_ids: string[]
}

export interface ShowcaseImageReorder {
  image_ids: string[]
}

export interface ShowcaseImageCheck {
  image_ids: string[]
}

export interface ShowcaseImageCheckResult {
  showcase_id: string
  existing_count: number
}
