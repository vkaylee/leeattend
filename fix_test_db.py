import re

files_to_fix = [
    "apiserver/tests/integration/attendance/import.rs",
    "apiserver/tests/integration/attendance/reprocess.rs",
    "apiserver/tests/integration/holiday/summary_holiday.rs",
    "apiserver/tests/integration/holiday/work_policy.rs",
    "apiserver/tests/integration/leave/api_upload.rs",
    "apiserver/tests/integration/payroll/traceability.rs",
    "apiserver/tests/integration/misc/worker_pool.rs"
]

for file in files_to_fix:
    with open(file, "r") as f:
        content = f.read()

    # Change _test_db to test_db
    content = re.sub(r'let \(_test_db, factory, app_state, _router\)', r'let (test_db, factory, app_state, _router)', content)
    content = re.sub(r'let pool = _test_db\.pool\(\)', r'let pool = test_db.pool()', content)
    content = re.sub(r'let \(_test_db, factory, _state, app\)', r'let (test_db, factory, _state, app)', content)

    # If test_db is not in scope, there might be a test where they don't instantiate test_db?
    # Let's see what happens if we replace `&test_db.url()` with `&std::env::var("DATABASE_URL").unwrap()` in those where test_db is really not defined.
    # Actually wait! What if TestDatabase is not created at all in those tests?
    
    with open(file, "w") as f:
        f.write(content)

