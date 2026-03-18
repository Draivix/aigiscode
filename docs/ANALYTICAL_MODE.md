# Analytical Mode

The legacy Python analytical-mode flow has been removed from the repository.

The current Rust CLI exposes deterministic analysis and architecture-surface
generation only:

```bash
aigiscode analyze /path/to/project
aigiscode surface /path/to/project
```

If analytical or policy-tuning workflows return, they should be implemented as
Rust-native features rather than restored as Python-side tooling.
