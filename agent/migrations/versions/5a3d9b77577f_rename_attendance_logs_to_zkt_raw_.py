"""rename attendance_logs to zkt_raw_attendance

Revision ID: 5a3d9b77577f
Revises: a5e4fad14fd9
Create Date: 2026-01-07 09:48:31.668199

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


# revision identifiers, used by Alembic.
revision: str = '5a3d9b77577f'
down_revision: Union[str, Sequence[str], None] = 'a5e4fad14fd9'
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # Rename table
    op.rename_table('attendance_logs', 'zkt_raw_attendance')
    
    # Rename indexes (Postgres specific)
    op.execute('ALTER INDEX ix_attendance_logs_device_id RENAME TO ix_zkt_raw_attendance_device_id')
    op.execute('ALTER INDEX ix_attendance_logs_user_id RENAME TO ix_zkt_raw_attendance_user_id')
    
    # Add index for timestamp which was missing in the first migration but added in model
    op.create_index(op.f('ix_zkt_raw_attendance_timestamp'), 'zkt_raw_attendance', ['timestamp'], unique=False)


def downgrade() -> None:
    op.drop_index(op.f('ix_zkt_raw_attendance_timestamp'), table_name='zkt_raw_attendance')
    op.execute('ALTER INDEX ix_zkt_raw_attendance_user_id RENAME TO ix_attendance_logs_user_id')
    op.execute('ALTER INDEX ix_zkt_raw_attendance_device_id RENAME TO ix_attendance_logs_device_id')
    op.rename_table('zkt_raw_attendance', 'attendance_logs')
