err_lines = [
    "    import json; cov=json.load(open('/workspace/.test_logs/coverage.json')); print('\\n\\n' + '='*60 + '\\n🤖 AI COVERAGE SUMMARY (Target: 50.0%)\\n' + '='*60); files=[f for f in cov['data'][0]['files'] if f['summary']['lines']['count'] > 0 and (f['summary']['lines']['covered'] / f['summary']['lines']['count']) < 0.5]; files.sort(key=lambda x: x['summary']['lines']['covered'] / x['summary']['lines']['count']); for f in files: print('❌ %s (%.1f%%)' % (f['filename'], (f['summary']['lines']['covered']/f['summary']['lines']['count'])*100)); if not files: print('✅ Tất cả các file đều đã đạt >= 50% coverage!'); print('='*60 + '\\n')",
    "                                                                                                                                                                                                                                                                                                                                                                                                                             ^^^"
]
print("Index:", err_lines[1].find('^'))
print("Char at index:", err_lines[0][err_lines[1].find('^'):])
