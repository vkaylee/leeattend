# 📚 LeeAttend Documentation Map

Welcome to the project documentation. To maintain clarity, please follow this structure when adding or searching for documents.

---

## 🏗️ 1. Technical Documentation (`/docs`)
*Target Audience: Developers & AI Agents*

- **[Codebase Summary](./codebase-summary.md)**: High-level overview of the project structure and tech stack.
- **[System Architecture](./system-architecture.md)**: Diagrams and descriptions of the backend/frontend interaction.
- **[API Reference](./api-reference.md)**: Documentation for REST and WebSocket endpoints.
- **[Code Standards](./code-standards.md)**: Mandatory coding patterns and naming conventions.
- **[Deployment Guide](./deployment-guide.md)**: Instructions for production and staging environments.
- **[UI/UX Standards](./ui-ux-standards.md)**: Design system and styling rules.

### Sub-folders:
- **[/infrastructure](./infrastructure/)**: Documentation for agents, workers, and hardware integration.
- **[/research](./research/)**: Deep dives into technical issues (e.g., Redis TTL analysis).
- **[/features](./features/)**: Technical specifications for specific feature implementations.
- **[/archive](./archive/)**: Deprecated or old documentation for historical reference.

---

## 💼 2. Business & Domain (`/documents`)
*Target Audience: Product Owners & Stakeholders*

- **[/business](../documents/business/)**: PRDs, workflows, and business glossaries.
- **[/audit](../documents/audit/)**: Compliance, security audits, and data flow analysis.
- **Knowledge Base**: Domain-specific logic located in the root of `/documents`.

---

## 🗺️ 3. Execution & Roadmap (`/plans`)
- Contains timestamped implementation plans for every major feature.

---

## 🚦 4. Global Governance
- **[PROJECT_OS.md](../PROJECT_OS.md)**: The "Constitution" containing mandatory WSL/RAM protocols and testing mandates. **Read this first!**

---

### 🤖 Note for AI Agents
If you create a new technical document, place it in the appropriate `/docs` sub-folder and update this map. Keep files under 200 lines by modularizing content.
