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


## 🛠️ Prerequisites

- **Python 3.10+** (For Agent)
- **Rust 1.75+** (For API Server)

## 📖 Documentation
- [System Architecture](docs/system-architecture.md)
- [API Reference](docs/api-reference.md)
- [Deployment Guide](docs/deployment-guide.md)

## 📝 Changelog
See [CHANGELOG.md](./CHANGELOG.md) for version history.
