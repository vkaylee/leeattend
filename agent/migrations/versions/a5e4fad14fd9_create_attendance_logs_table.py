"""create attendance logs table

Revision ID: a5e4fad14fd9
Revises: 
Create Date: 2026-01-07 08:49:57.314174

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa


# revision identifiers, used by Alembic.
revision: str = 'a5e4fad14fd9'
down_revision: Union[str, Sequence[str], None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    """Upgrade schema."""
    op.create_table(
        'attendance_logs',
        sa.Column('id', sa.Integer(), autoincrement=True, nullable=False),
        sa.Column('device_id', sa.String(length=50), nullable=False),
        sa.Column('user_id', sa.String(length=50), nullable=False),
        sa.Column('timestamp', sa.DateTime(), nullable=False),
        sa.Column('status', sa.Integer(), nullable=True),
        sa.Column('punch', sa.Integer(), nullable=True),
        sa.Column('created_at', sa.DateTime(), server_default=sa.text('now()'), nullable=True),
        sa.PrimaryKeyConstraint('id')
    )
    op.create_index(op.f('ix_attendance_logs_device_id'), 'attendance_logs', ['device_id'], unique=False)
    op.create_index(op.f('ix_attendance_logs_user_id'), 'attendance_logs', ['user_id'], unique=False)


def downgrade() -> None:
    """Downgrade schema."""
    op.drop_index(op.f('ix_attendance_logs_user_id'), table_name='attendance_logs')
    op.drop_index(op.f('ix_attendance_logs_device_id'), table_name='attendance_logs')
    op.drop_table('attendance_logs')
