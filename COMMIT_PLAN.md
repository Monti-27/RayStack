# RayStack Incremental Commit Plan

## Strategy
We will reconstruct the codebase in 13 steps using a bash script. The script will:
1. Clear the current `src` directory (after backing it up or assuming git handles valid state).
2. Create files with partial content.
3. Commit.
4. Append/Modify content.
5. Commit again.

## Commits
1.  **Project Init**: `Cargo.toml`, `.gitignore`, `.env` (template).
2.  **Engine Config**: `src/engine/mod.rs` (struct definitions).
3.  **Notifier System**: `src/notifier.rs` (initial struct & impl).
4.  **Listener Scaffold**: `src/engine/listener.rs` (imports & function signature).
5.  **Listener Connection**: `src/engine/listener.rs` (add connection logic).
6.  **Listener Subscription**: `src/engine/listener.rs` (add subscription & loop).
7.  **Listener Logic**: `src/engine/listener.rs` (add filtering logic).
8.  **Handler Scaffold**: `src/engine/handler.rs` (imports & signature).
9.  **Handler Parsing**: `src/engine/handler.rs` (add parsing logic).
10. **Handler Notification**: `src/engine/handler.rs` (add notification logic).
11. **Main Entry**: `src/main.rs` (imports & setup).
12. **Main Logic**: `src/main.rs` (spawn tasks & loop).
13. **Final Polish**: Ensure everything compiles and is formatted.
