"""baseline

Mark the schema created by db/init/*.sql as the baseline. This revision is a
no-op upgrade and drops all tables on downgrade. Future schema changes should
be added as new revisions building on top of this baseline.

Revision ID: 0001_baseline
Revises:
Create Date: 2026-04-18 00:00:00
"""
from __future__ import annotations

from typing import Sequence, Union

# revision identifiers, used by Alembic.
revision: str = "0001_baseline"
down_revision: Union[str, Sequence[str], None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    """No-op: baseline schema is created by db/init/*.sql on first container start."""
    pass


def downgrade() -> None:
    """No downgrade for baseline."""
    pass
