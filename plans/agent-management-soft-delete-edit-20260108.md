# Implementation Plan: Agent Management (Soft Delete & Edit)

## Approach
This plan implements agent deletion as a "soft delete" (marking an agent inactive rather than removing it from the database) and agent editing via a dedicated modal/page. This approach was chosen for its benefits in data integrity, auditing, and user experience, allowing for a cleaner and more robust implementation compared to hard deletion or complex inline editing.

## Steps

#### **Phase 1: Backend - API Server (`apiserver` - Rust/Axum)**

1.  **Database Migration (Add `is_active` column)**
    *   Create a new migration file to add an `is_active` (boolean, default true) column to the `agents` table.
    *   **Files to create/modify:** `apiserver/migrations/[timestamp]_add_is_active_to_agents.sql`
    *   **Snippet (conceptual):**
        ```sql
        -- Up migration
        ALTER TABLE agents ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;

        -- Down migration
        ALTER TABLE agents DROP COLUMN is_active;
        ```
    *   Run the migration to apply changes to the database.

2.  **Update Agent Model (`models.rs`)**
    *   Add the `is_active` field to the `Agent` struct.
    *   **Files to modify:** `apiserver/src/models.rs`

3.  **Implement API Endpoints & Handlers (`handlers.rs`, `agent_manager.rs`)**
    *   **Delete Agent (Soft Delete):**
        *   Create a `DELETE /agents/{id}` endpoint.
        *   The handler will update the `is_active` field of the specified agent to `false` in the database.
        *   **Files to modify:** `apiserver/src/handlers.rs`, `apiserver/src/agent_manager.rs` (for DB logic).
        *   **Snippet (conceptual - `agent_manager.rs`):**
            ```rust
            pub async fn soft_delete_agent(pool: &PgPool, agent_id: Uuid) -> Result<(), Error> {
                sqlx::query!(
                    "UPDATE agents SET is_active = FALSE WHERE id = $1",
                    agent_id
                )
                .execute(pool)
                .await?;
                Ok(())
            }
            ```
    *   **Edit Agent:**
        *   Create a `PUT /agents/{id}` or `PATCH /agents/{id}` endpoint. `PATCH` is preferred for partial updates.
        *   Define an `UpdateAgent` DTO (Data Transfer Object) in `models.rs` that contains optional fields for editable attributes (e.g., `name`, `location`).
        *   The handler will update the specified agent's fields in the database based on the provided DTO.
        *   **Files to modify:** `apiserver/src/models.rs` (for DTO), `apiserver/src/handlers.rs`, `apiserver/src/agent_manager.rs` (for DB logic).
        *   **Snippet (conceptual - `agent_manager.rs`):**
            ```rust
            pub async fn update_agent(
                pool: &PgPool,
                agent_id: Uuid,
                update_data: UpdateAgent,
            ) -> Result<Agent, Error> {
                // Construct update query dynamically based on Option<T> fields in update_data
                // For simplicity, showing a direct update here
                sqlx::query_as!(
                    Agent,
                    "UPDATE agents SET name = COALESCE($1, name), location = COALESCE($2, location) WHERE id = $3 RETURNING *",
                    update_data.name,
                    update_data.location,
                    agent_id
                )
                .fetch_one(pool)
                .await
            }
            ```
    *   **Filter Agent Listing:**
        *   Modify the existing `GET /agents` endpoint to only return agents where `is_active = TRUE` by default. Provide an optional query parameter (e.g., `?include_inactive=true`) to fetch all agents if needed for admin purposes.
        *   **Files to modify:** `apiserver/src/agent_manager.rs`

4.  **Integrate Endpoints into Main Application (`main.rs`)**
    *   Add the new routes to the Axum router.
    *   **Files to modify:** `apiserver/src/main.rs`

5.  **Backend Testing (`tests/`)**
    *   Create new integration tests to verify the `DELETE /agents/{id}` (soft delete) and `PUT/PATCH /agents/{id}` endpoints.
    *   Ensure that `GET /agents` correctly filters out inactive agents.
    *   **Files to create:** `apiserver/tests/agent_crud.rs`

#### **Phase 2: Frontend - Web Dashboard (`webdashboard` - React/TypeScript)**

1.  **Update API Client (`api.ts`)**
    *   Add new functions for calling the delete and edit agent API endpoints.
    *   **Files to modify:** `webdashboard/src/lib/api.ts`
    *   **Snippet (conceptual):**
        ```typescript
        export const deleteAgent = async (id: string): Promise<void> => {
            await axios.delete(`/api/agents/${id}`);
        };

        export const updateAgent = async (id: string, data: Partial<Agent>): Promise<Agent> => {
            const response = await axios.patch(`/api/agents/${id}`, data);
            return response.data;
        };
        ```

2.  **Create Edit Agent Modal Component**
    *   Create a new React component for the agent edit form. This component will handle fetching an agent's details, pre-filling the form, input validation, and submitting updates.
    *   **Files to create:** `webdashboard/src/components/AgentEditModal.tsx`
    *   **Snippet (conceptual):**
        ```typescript jsx
        // webdashboard/src/components/AgentEditModal.tsx
        import React, { useState, useEffect } from 'react';
        // ... other imports for form fields, validation, etc.

        interface AgentEditModalProps {
            agentId: string | null;
            isOpen: boolean;
            onClose: () => void;
            onSave: (updatedAgent: Agent) => void;
        }

        const AgentEditModal: React.FC<AgentEditModalProps> = ({ agentId, isOpen, onClose, onSave }) => {
            const [name, setName] = useState('');
            const [location, setLocation] = useState('');
            // ... other agent fields

            useEffect(() => {
                if (isOpen && agentId) {
                    // Fetch agent details and populate form
                    // e.g., api.fetchAgent(agentId).then(agent => { setName(agent.name); setLocation(agent.location); });
                }
            }, [isOpen, agentId]);

            const handleSubmit = async () => {
                // Validation logic
                if (!name.trim()) {
                    alert('Agent name cannot be empty');
                    return;
                }
                try {
                    const updatedAgent = await api.updateAgent(agentId!, { name, location });
                    onSave(updatedAgent);
                    onClose();
                } catch (error) {
                    console.error('Failed to update agent:', error);
                    // Handle error, show toast
                }
            };

            return (
                <Dialog open={isOpen} onClose={onClose}>
                    <DialogTitle>{agentId ? 'Edit Agent' : 'Create Agent'}</DialogTitle>
                    <DialogContent>
                        <TextField label="Agent Name" value={name} onChange={(e) => setName(e.target.value)} />
                        <TextField label="Location" value={location} onChange={(e) => setLocation(e.target.value)} />
                        {/* ... other fields */}
                    </DialogContent>
                    <DialogActions>
                        <Button onClick={onClose}>Cancel</Button>
                        <Button onClick={handleSubmit} color="primary">Save</Button>
                    </DialogActions>
                </Dialog>
            );
        };

        export default AgentEditModal;
        ```

3.  **Update Agents Page (`AgentsPage.tsx`)**
    *   Add "Edit" and "Delete" buttons/actions to each agent row/card.
    *   Implement state to manage the visibility and data for the `AgentEditModal`.
    *   Implement a confirmation dialog for deletion.
    *   Integrate the `AgentEditModal` component.
    *   Refresh the agent list after successful edit or delete.
    *   **Files to modify:** `webdashboard/src/pages/AgentsPage.tsx`
    *   **Snippet (conceptual - `AgentsPage.tsx`):**
        ```typescript jsx
        // webdashboard/src/pages/AgentsPage.tsx
        import React, { useState, useEffect } from 'react';
        import AgentEditModal from '../components/AgentEditModal';
        // ... other imports for agent listing, dialogs, etc.

        const AgentsPage: React.FC = () => {
            const [agents, setAgents] = useState<Agent[]>([]);
            const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
            const [isEditModalOpen, setIsEditModalOpen] = useState(false);

            const fetchAgents = async () => {
                // Fetch agents, filtering for is_active = true
                const data = await api.fetchAgents(); // Assuming API call can take ?include_inactive=true
                setAgents(data);
            };

            useEffect(() => {
                fetchAgents();
            }, []);

            const handleEditClick = (agentId: string) => {
                setSelectedAgentId(agentId);
                setIsEditModalOpen(true);
            };

            const handleDeleteClick = async (agentId: string) => {
                if (window.confirm('Are you sure you want to delete this agent?')) {
                    try {
                        await api.deleteAgent(agentId);
                        fetchAgents(); // Refresh list
                    } catch (error) {
                        console.error('Failed to delete agent:', error);
                        // Show error message
                    }
                }
            };

            const handleModalSave = (updatedAgent: Agent) => {
                setAgents(agents.map(agent => agent.id === updatedAgent.id ? updatedAgent : agent));
            };

            return (
                <div>
                    <h1>Agents</h1>
                    {/* Render agent list, with Edit and Delete buttons */}
                    {agents.map(agent => (
                        <div key={agent.id}>
                            <span>{agent.name} - {agent.location}</span>
                            <button onClick={() => handleEditClick(agent.id)}>Edit</button>
                            <button onClick={() => handleDeleteClick(agent.id)}>Delete</button>
                        </div>
                    ))}

                    <AgentEditModal
                        agentId={selectedAgentId}
                        isOpen={isEditModalOpen}
                        onClose={() => setIsEditModalOpen(false)}
                        onSave={handleModalSave}
                    />
                </div>
            );
        };

        export default AgentsPage;
        ```

4.  **Frontend Testing (`tests/`)**
    *   Add new unit/integration tests for `AgentEditModal` component.
    *   Add integration tests for `AgentsPage.tsx` to simulate edit/delete actions and verify UI updates and API calls.
    *   **Files to create:** `webdashboard/src/pages/AgentsPage.test.tsx`, `webdashboard/src/components/AgentEditModal.test.tsx`

## Timeline

| Phase | Duration (Estimate) |
| :--------------------------------- | :------------------ |
| **Backend**                        |                     |
| Database Migration                 | 1 hr                |
| Model Update                       | 0.5 hr              |
| API Endpoints & Handlers           | 4 hr                |
| Integrate Routes                   | 0.5 hr              |
| Backend Testing                    | 3 hr                |
| **Frontend**                       |                     |
| Update API Client                  | 1 hr                |
| Create Edit Agent Modal Component  | 4 hr                |
| Update Agents Page                 | 4 hr                |
| Frontend Testing                   | 3 hr                |
| **Total**                          | **21 hours**        |

## Rollback Plan
1.  **Backend:**
    *   Revert the database migration (down migration) to remove the `is_active` column.
    *   Revert code changes in `apiserver/src/models.rs`, `apiserver/src/handlers.rs`, `apiserver/src/agent_manager.rs`, `apiserver/src/main.rs`.
    *   Revert test files.
2.  **Frontend:**
    *   Revert code changes in `webdashboard/src/lib/api.ts`, `webdashboard/src/pages/AgentsPage.tsx`.
    *   Remove `webdashboard/src/components/AgentEditModal.tsx` and associated test files.

## Security Checklist
*   [x] **Input validation:** Ensure all incoming data to the API (for agent updates) is thoroughly validated on the server-side to prevent malicious input (e.g., SQL injection, XSS if data is reflected).
*   [x] **Authentication & Authorization checks:** Verify that only authenticated and authorized users (e.g., administrators) can perform edit and delete operations on agents. Implement proper middleware or guards for API endpoints.
*   [ ] **Rate limiting:** Consider adding rate limiting to the edit/delete endpoints to prevent abuse (though lower priority for internal tools).
*   [x] **Error handling:** Implement robust error handling on both frontend and backend to gracefully manage failures and provide informative messages without exposing sensitive details.
*   [x] **Data integrity:** Ensure that deleting an agent (soft delete) doesn't inadvertently break references or cause inconsistencies in other parts of the system.
