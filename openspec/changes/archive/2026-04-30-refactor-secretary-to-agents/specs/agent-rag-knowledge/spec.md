## ADDED Requirements

### Requirement: Per-Agent RAG ingestion
The system SHALL ingest `rag_knowledge.md` for each valid Agent plugin when RAG is enabled in the manifest and runtime config.

#### Scenario: RAG knowledge exists
- **WHEN** a valid plugin with RAG enabled contains `rag_knowledge.md`
- **THEN** the system chunks and indexes the file for that plugin

#### Scenario: RAG disabled
- **WHEN** a plugin has RAG disabled
- **THEN** the system does not index or retrieve that plugin's `rag_knowledge.md`

### Requirement: sqlite-vec indexing
The system SHALL store Agent knowledge embeddings in SQLite using sqlite-vec with plugin ID, plugin version, source hash, chunk text, embedding model, embedding dimension, and vector metadata.

#### Scenario: Embedding dimension matches
- **WHEN** the embedding vector dimension matches the plugin config and sqlite-vec table
- **THEN** the system stores the chunk vector and marks the index usable

#### Scenario: Embedding dimension mismatches
- **WHEN** an embedding vector dimension does not match the expected dimension
- **THEN** the system rejects the index update and reports the mismatch

### Requirement: Retrieval during Agent prompt assembly
The system SHALL retrieve top matching chunks from the active Agent's own RAG index and include them in prompt assembly with source labels.

#### Scenario: Relevant chunks are found
- **WHEN** the user sends a message and the active Agent has matching RAG chunks
- **THEN** the system includes up to the configured `rag_top_k` chunks in that Agent's prompt

#### Scenario: No chunks are found
- **WHEN** no relevant chunks exist for an Agent
- **THEN** the system sends the Agent prompt without RAG snippets and continues the conversation

### Requirement: RAG data isolation
The system SHALL prevent one Agent from retrieving another Agent's private RAG chunks unless a future explicit sharing feature is added.

#### Scenario: Two Agents have indexes
- **WHEN** Agent A responds to a user message
- **THEN** the system retrieves only chunks indexed for Agent A

### Requirement: Rebuild and cleanup
The system SHALL rebuild RAG indexes when plugin knowledge changes and delete plugin RAG indexes when a plugin is uninstalled.

#### Scenario: Knowledge file changes
- **WHEN** `rag_knowledge.md` hash changes for an installed plugin
- **THEN** the system marks the plugin RAG index stale and rebuilds it on user request or configured refresh

#### Scenario: Plugin is uninstalled
- **WHEN** the user uninstalls a plugin
- **THEN** the system deletes that plugin's RAG chunks and vector rows
