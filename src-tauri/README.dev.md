
## Regenerating IPC types

After changing any Rust type that derives `ts_rs::TS`:

```bash
pnpm gen-types
```

Generated files land in `src/ipc/generated/`. Commit them.
