import os
import urllib.parse
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from dotenv import load_dotenv

load_dotenv()

PG_DATABASE_ADDR = os.getenv("PG_DATABASE_ADDR")
PG_DATABASE_PORT = os.getenv("PG_DATABASE_PORT")
PG_DATABASE_DB = os.getenv("PG_DATABASE_DB")
PG_DATABASE_USER = os.getenv("PG_DATABASE_USER")
PG_DATABASE_PASS = os.getenv("PG_DATABASE_PASS")

if PG_DATABASE_PASS:
    encoded_pass = urllib.parse.quote_plus(PG_DATABASE_PASS)
else:
    encoded_pass = ""

SQLALCHEMY_DATABASE_URL = f"postgresql://{PG_DATABASE_USER}:{encoded_pass}@{PG_DATABASE_ADDR}:{PG_DATABASE_PORT}/{PG_DATABASE_DB}"

engine = create_engine(SQLALCHEMY_DATABASE_URL)
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()
