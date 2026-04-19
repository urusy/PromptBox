FROM python:3.14-slim

WORKDIR /app

# システム依存パッケージ
RUN apt-get update && apt-get install -y \
    libpq-dev \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Python依存パッケージ
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# アプリケーションコード
COPY . .

# エントリポイントに実行権限
RUN chmod +x /app/scripts/entrypoint.sh

# 非rootユーザーで実行
RUN useradd -m appuser && chown -R appuser:appuser /app
USER appuser

# ポート公開
EXPOSE 8000

# 起動コマンド（alembic migrate → uvicorn）
CMD ["/app/scripts/entrypoint.sh"]