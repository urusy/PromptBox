# 設計: 検索実行時にフィルターパネルを閉じる（issue #31）

作成日: 2026-08-13 / 対象: [#31](../../issues/31) / ステータス: **実装済み（2026-08-13）**

**実装**: `SearchForm.tsx` の `handleSubmit` に `setIsExpanded(false)` を追加（下記の設計どおり1行）。
**検証**: `tsc --noEmit` / `eslint --max-warnings 0` / `vite build` 通過。実機確認は未実施。

## 要求

一覧（ギャラリー）で Search ボタンを押したとき、開いているフィルターパネルを閉じる。

## 現状の観測（2026-08-13）

- パネルの開閉は `frontend/src/components/gallery/SearchForm.tsx` の
  ローカル state `isExpanded`（`SearchForm.tsx:176`）が持つ。
  - トグルは「Filters」ボタン（`SearchForm.tsx:558`）のみ。
  - パネル本体は `{isExpanded && ...}`（`SearchForm.tsx:574`）。
- 検索の実行は `handleSubmit`（`SearchForm.tsx:354-357`）。
  `type="submit"` の Search ボタン（`SearchForm.tsx:567`）と、
  検索入力での Enter の両方がここに入る。
- 検索実行後もパネルは開いたままになる — これが issue の症状。

## 設計

`handleSubmit` に 1 行足すだけ。

```tsx
const handleSubmit = (e: React.FormEvent) => {
  e.preventDefault()
  onSearch({ ...localParams, page: 1 })
  setIsExpanded(false)   // ← 追加
}
```

### 境界の整理（意図的に閉じないもの）

| 操作 | 閉じるか | 理由 |
|---|---|---|
| Search ボタン | **閉じる** | issue の要求そのもの |
| 検索入力での Enter | **閉じる** | 同じ `handleSubmit` を通る。挙動が割れる方が不自然 |
| Reset（× ボタン、`handleReset:359`） | 閉じない | 条件を消してから組み直す動線を壊さない |
| プリセット選択（`handleSelectPreset:419`） | 閉じない | パネルが開いていれば「プリセットで何が適用されたか」が見える方が有益 |

## 検証

- フロントにテストファイルは存在しない（Vitest 環境のみ）→ ゲートは
  `npx tsc --noEmit` / `npm run lint` / `npm run build`。
- 実機: パネルを開いて Search → 閉じること、Enter でも閉じること、
  Filters ボタンで再度開けること。モバイル幅（パネルが画面を占有する状況）で特に確認。

## 工数

XS（1行 + 検証）。
