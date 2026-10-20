# PASS 2 – VERTICAL (entry point → full call chain)

This batch contains one or more **entry points** (HTTP route handlers, console commands, queue jobs, event listeners, cron tasks) and the complete transitive set of code they call, as resolved by the static call graph.

Walk EVERY entry point top to bottom as if you were stepping through it in a debugger. For each step ask:

1. **Is this branch reachable?** Check preceding guards, early returns, exceptions already thrown, type constraints. Report `dead_branch`.
2. **Does this layer do anything?** A controller that calls a service that calls a manager that calls a repository – if any of these only forwards arguments, report `redundant_layer`.
3. **Is this check repeated?** The same `if (!$user)`, the same `assertValid`, the same null-guard on the same value more than once along the path → `repeated_validation`. Cite every occurrence.
4. **Is every parameter used?** For each function on the path, check every parameter is read. → `unused_param`.
5. **Is every injected dependency used?** Constructor-injected services never referenced → `unused_dependency`.
6. **Catch blocks:** catches that swallow and do nothing, or re-throw the identical exception → `dead_branch`.
7. **Is anything on this path a copy of something else on this path?** Two helpers in the chain doing the same transformation → `clone_*`.
8. **Does anything here reimplement something in `shared_signatures`?** If a local helper does what a shared utility clearly does, report `clone_semantic` with `canonical_candidate` = the shared utility.

Then, for each entry point, produce in `path_summary` a one-paragraph description of what it actually does end to end in plain words, and list `layers_traversed` (count). This summary feeds the domain pass later – write it so that someone could spot a second entry point doing the same business thing.

Do not stop at the first finding in a function. Exhaust the function, then move on.

Remember: zero findings in a chain longer than 5 layers is a red flag about your work, not about the code.
