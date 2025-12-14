# バックアップ戦略

このドキュメントでは、Prompt Boxのデータバックアップ戦略について説明します。

## 概要

Prompt Boxは以下のデータを管理しており、それぞれ適切なバックアップが必要です：

| データ種別 | 保存場所 | 重要度 | バックアップ頻度 |
|-----------|---------|--------|-----------------|
| データベース | PostgreSQL | 高 | 日次 |
| 元画像 | storage/original/ | 高 | 週次/差分 |
| サムネイル | storage/thumbnail/ | 低 | 不要（再生成可能） |
| 設定ファイル | .env, docker-compose.yml | 高 | 変更時 |

## データベースバックアップ

### 日次バックアップ（推奨）

```bash
#!/bin/bash
# backup_db.sh

BACKUP_DIR="/backup/db"
DATE=$(date +%Y%m%d_%H%M%S)
CONTAINER_NAME="promptbox-db-1"

# バックアップディレクトリ作成
mkdir -p ${BACKUP_DIR}

# pg_dump実行
docker exec ${CONTAINER_NAME} pg_dump -U ${DB_USER} -d ${DB_NAME} -F c -f /tmp/backup.dump
docker cp ${CONTAINER_NAME}:/tmp/backup.dump ${BACKUP_DIR}/promptbox_${DATE}.dump

# 古いバックアップを削除（7日以上前）
find ${BACKUP_DIR} -name "promptbox_*.dump" -mtime +7 -delete

echo "Database backup completed: ${BACKUP_DIR}/promptbox_${DATE}.dump"
```

### リストア手順

```bash
#!/bin/bash
# restore_db.sh

BACKUP_FILE=$1
CONTAINER_NAME="promptbox-db-1"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: restore_db.sh <backup_file>"
    exit 1
fi

# バックアップファイルをコンテナにコピー
docker cp ${BACKUP_FILE} ${CONTAINER_NAME}:/tmp/restore.dump

# リストア実行
docker exec ${CONTAINER_NAME} pg_restore -U ${DB_USER} -d ${DB_NAME} -c /tmp/restore.dump

echo "Database restore completed from: ${BACKUP_FILE}"
```

### cronジョブ設定

```bash
# crontab -e
# 毎日午前3時にバックアップ実行
0 3 * * * /path/to/backup_db.sh >> /var/log/promptbox_backup.log 2>&1
```

## 画像ストレージバックアップ

### rsyncによる差分バックアップ（推奨）

```bash
#!/bin/bash
# backup_images.sh

SOURCE_DIR="/path/to/promptbox/storage/original"
BACKUP_DIR="/backup/images"
DATE=$(date +%Y%m%d)

# 差分バックアップ
rsync -avz --delete \
    --backup --backup-dir="${BACKUP_DIR}/incremental/${DATE}" \
    ${SOURCE_DIR}/ ${BACKUP_DIR}/current/

echo "Image backup completed to: ${BACKUP_DIR}/current/"
```

### rcloneによるクラウドバックアップ（オプション）

```bash
#!/bin/bash
# backup_to_cloud.sh

# rclone設定が必要
# rclone config で事前にリモートを設定

rclone sync /path/to/promptbox/storage/original remote:promptbox-backup/images \
    --progress \
    --transfers 4 \
    --exclude "*.tmp"

echo "Cloud backup completed"
```

## バックアップ保持ポリシー

### 推奨保持期間

| バックアップ種別 | 保持期間 | 世代数 |
|-----------------|---------|--------|
| 日次DB | 7日 | 7 |
| 週次DB | 4週間 | 4 |
| 月次DB | 12ヶ月 | 12 |
| 画像差分 | 30日 | 30 |
| 画像フル | 無期限 | 1（最新） |

### ストレージ容量見積もり

```
データベース:
- 1万枚の画像: 約50MB
- 日次バックアップ × 7日 = 350MB
- 月次バックアップ × 12ヶ月 = 600MB

画像ストレージ:
- 1万枚 × 平均5MB = 50GB
- 差分バックアップ（月10%変更）= 5GB/月
```

## 災害復旧（DR）計画

### RPO/RTO目標

| 指標 | 目標 | 説明 |
|------|------|------|
| RPO（Recovery Point Objective） | 24時間 | 最大24時間分のデータ損失を許容 |
| RTO（Recovery Time Objective） | 4時間 | 4時間以内に復旧完了 |

### 復旧手順

1. **新環境の準備**（30分）
   ```bash
   git clone <repository>
   cd promptbox
   cp .env.example .env
   # .env を編集
   ```

2. **Dockerコンテナ起動**（10分）
   ```bash
   docker compose up -d db
   # DBが起動するまで待機
   sleep 30
   ```

3. **データベースリストア**（30分）
   ```bash
   ./restore_db.sh /backup/db/latest.dump
   ```

4. **画像リストア**（1-3時間、データ量依存）
   ```bash
   rsync -avz /backup/images/current/ ./storage/original/
   ```

5. **アプリケーション起動**（10分）
   ```bash
   docker compose up -d
   ```

6. **動作確認**（30分）
   - ログイン確認
   - 画像表示確認
   - メタデータ表示確認

## 自動バックアップスクリプト

### 統合バックアップスクリプト

```bash
#!/bin/bash
# full_backup.sh
# 完全バックアップを実行

set -e

BACKUP_ROOT="/backup/promptbox"
DATE=$(date +%Y%m%d_%H%M%S)
LOG_FILE="/var/log/promptbox_backup.log"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a ${LOG_FILE}
}

log "Starting full backup..."

# 1. データベースバックアップ
log "Backing up database..."
docker exec promptbox-db-1 pg_dump -U ${DB_USER} -d ${DB_NAME} -F c > ${BACKUP_ROOT}/db/promptbox_${DATE}.dump
log "Database backup completed"

# 2. 画像バックアップ（差分）
log "Backing up images..."
rsync -avz --delete ./storage/original/ ${BACKUP_ROOT}/images/current/
log "Image backup completed"

# 3. 設定ファイルバックアップ
log "Backing up configuration..."
cp .env ${BACKUP_ROOT}/config/.env.${DATE}
cp docker-compose.yml ${BACKUP_ROOT}/config/docker-compose.yml.${DATE}
log "Configuration backup completed"

# 4. 古いバックアップを削除
log "Cleaning old backups..."
find ${BACKUP_ROOT}/db -name "*.dump" -mtime +7 -delete
find ${BACKUP_ROOT}/config -name ".env.*" -mtime +30 -delete
log "Cleanup completed"

log "Full backup completed successfully"
```

## 監視とアラート

### バックアップ監視項目

- [ ] バックアップジョブの成功/失敗
- [ ] バックアップファイルのサイズ（異常な増減の検知）
- [ ] ストレージ使用量
- [ ] 最終バックアップからの経過時間

### ヘルスチェックスクリプト

```bash
#!/bin/bash
# check_backup_health.sh

BACKUP_DIR="/backup/promptbox/db"
MAX_AGE_HOURS=25  # 25時間以内のバックアップが必要

# 最新のバックアップファイルを確認
LATEST=$(find ${BACKUP_DIR} -name "*.dump" -mmin -$((MAX_AGE_HOURS * 60)) | head -1)

if [ -z "$LATEST" ]; then
    echo "ERROR: No backup found within ${MAX_AGE_HOURS} hours"
    exit 1
fi

# ファイルサイズ確認（最小10KB）
SIZE=$(stat -f%z "$LATEST" 2>/dev/null || stat -c%s "$LATEST")
if [ "$SIZE" -lt 10240 ]; then
    echo "ERROR: Backup file too small: ${SIZE} bytes"
    exit 1
fi

echo "OK: Latest backup is ${LATEST} (${SIZE} bytes)"
exit 0
```

## セキュリティ考慮事項

### バックアップデータの保護

1. **暗号化**
   ```bash
   # GPGで暗号化
   gpg --symmetric --cipher-algo AES256 backup.dump
   ```

2. **アクセス制限**
   ```bash
   chmod 600 /backup/promptbox/*
   chown root:root /backup/promptbox/*
   ```

3. **転送時の暗号化**
   - rsync over SSH使用
   - rclone crypt backend使用

### 機密データの取り扱い

- `.env`ファイルにはパスワードハッシュ、SECRET_KEYが含まれる
- バックアップ先でも適切なアクセス制限を設定

## テスト計画

### 定期的な復旧テスト

| テスト種別 | 頻度 | 内容 |
|-----------|------|------|
| バックアップ整合性確認 | 週次 | バックアップファイルの破損チェック |
| 部分復旧テスト | 月次 | DBのみ、または画像のみの復旧 |
| 完全復旧テスト | 四半期 | 新規環境での完全復旧 |

### 復旧テストチェックリスト

- [ ] バックアップファイルからDBリストア成功
- [ ] 全画像が正常に表示される
- [ ] メタデータが正しく読み取れる
- [ ] ログイン可能
- [ ] 新規画像のインポートが動作する

## 参考リンク

- [PostgreSQL Backup and Restore](https://www.postgresql.org/docs/current/backup.html)
- [rsync Documentation](https://rsync.samba.org/documentation.html)
- [rclone Documentation](https://rclone.org/docs/)
