import os
import urllib.parse
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from dotenv import load_dotenv

load_dotenv()

# Ưu tiên DATABASE_URL từ .env (hỗ trợ cả SQLite và PostgreSQL)
SQLALCHEMY_DATABASE_URL = os.getenv("DATABASE_URL")

# Fallback sang cấu hình PostgreSQL chi tiết nếu DATABASE_URL không tồn tại
if not SQLALCHEMY_DATABASE_URL:
    PG_DATABASE_ADDR = os.getenv("PG_DATABASE_ADDR")
    PG_DATABASE_PORT = os.getenv("PG_DATABASE_PORT")
    PG_DATABASE_DB = os.getenv("PG_DATABASE_DB")
    PG_DATABASE_USER = os.getenv("PG_DATABASE_USER")
    PG_DATABASE_PASS = os.getenv("PG_DATABASE_PASS")

    if PG_DATABASE_PASS:
        encoded_pass = urllib.parse.quote_plus(PG_DATABASE_PASS)
    else:
        encoded_pass = ""

    if PG_DATABASE_USER and PG_DATABASE_ADDR:
        SQLALCHEMY_DATABASE_URL = f"postgresql://{PG_DATABASE_USER}:{encoded_pass}@{PG_DATABASE_ADDR}:{PG_DATABASE_PORT}/{PG_DATABASE_DB}"
    else:
        # Default fallback to local sqlite if nothing else is configured
        SQLALCHEMY_DATABASE_URL = "sqlite:///./attendance.db"

# SQLite requires special args for multi-threading
connect_args = {"check_same_thread": False} if SQLALCHEMY_DATABASE_URL.startswith("sqlite") else {}

engine = create_engine(SQLALCHEMY_DATABASE_URL, connect_args=connect_args)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

# Tự động tạo bảng nếu chưa có (Hữu ích cho SQLite/Cài đặt ban đầu)
from models import Base
Base.metadata.create_all(bind=engine)

def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
