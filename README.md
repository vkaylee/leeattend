# LeeAttend - Attendance Management System

A comprehensive attendance management system comprising a local agent for ZKTeco devices and a central high-performance API server.

## 📂 Project Structure

```
leeattend/
├── agent/            # Python Client (Runs on local machine/site)
│   ├── Connects to ZKTeco XFace100 devices
│   └── Syncs data to Central Server
│
├── apiserver/        # Rust API Server (Runs on Central Cloud/Server)
│   ├── High-performance REST API (Axum)
│   └── Central Database Management
│
└── docs/             # System Documentation
```

## Dự án bao gồm các thành phần:

1.  **apiserver**: Viết bằng **Rust (Axum)**, chịu trách nhiệm quản lý Agent, nhận log và cung cấp API cho Dashboard.
2.  **agent**: Viết bằng **Python (FastAPI/PyZK)**, chạy tại local để kết nối máy chấm công ZKTeco.
3.  **webdashboard**: Viết bằng **Next.js**, giao diện web để quản lý và theo dõi hệ thống.
4.  **docs**: Tài liệu hướng dẫn chi tiết.


## 🤖 For AI Agents: How to Debug UI

If you need to verify UI changes or fix loading issues:

1.  **Ensure Backend is running**: `cd apiserver && cargo run`
2.  **Ensure Frontend is running**: `cd webdashboard && npm run dev`
3.  **Run Automated Debug**:
    ```bash
    cd webdashboard
    npx playwright test tests/debug-ui.spec.ts --reporter=line
    ```
4.  **Analyze**: Look for `[BROWSER]` logs in the output to see what's happening inside the browser.


## 📖 Documentation
- [System Architecture](docs/system-architecture.md)
- [API Reference](docs/api-reference.md)
- [Deployment Guide](docs/deployment-guide.md)

## 📝 Changelog
See [CHANGELOG.md](./CHANGELOG.md) for version history.
