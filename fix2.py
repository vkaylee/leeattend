import re

def fix_file(file, pattern, replacement):
    with open(file, "r") as f:
        content = f.read()
    new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    if new_content != content:
        with open(file, "w") as f:
            f.write(new_content)

fix_file(
    "apiserver/tests/integration/payroll/traceability.rs",
    r'    let factory = TestFactory::new\(pool.clone\(\)\);\n    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test\(&test_db.url\(\)\);',
    r'    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&db.url());\n    let factory = TestFactory::new(pool.clone());'
)

fix_file(
    "apiserver/tests/integration/misc/worker_pool.rs",
    r'    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test\(&test_db.url\(\)\);\n    let \(test_db, factory, state, _router\) = setup_app_tower\(\).await;',
    r'    let (test_db, factory, state, _router) = setup_app_tower().await;\n    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());'
)

fix_file(
    "apiserver/tests/integration/holiday/work_policy.rs",
    r'    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test\(&test_db.url\(\)\);\n    let \(test_db, factory, _state, _app\) = setup_app_tower\(\).await;',
    r'    let (test_db, factory, _state, _app) = setup_app_tower().await;\n    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());'
)

fix_file(
    "apiserver/tests/integration/holiday/work_policy.rs",
    r'    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test\(&test_db.url\(\)\);\n    let \(address, test_db\) = spawn_app\(\).await;',
    r'    let (address, test_db) = spawn_app().await;\n    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());'
)

