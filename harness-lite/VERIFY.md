# Harness Lite — tests and checks

Read this before writing tests or choosing what to run. It extends the [shared contract](CONTRACT.md).

## Writing tests

Each test protects an observable behavior or invariant against a credible regression that existing coverage misses; extend cases rather than duplicate a proof, and avoid tautologies, copies of source shape and production seams added only for a test. For bug fixes, show fail-before/pass-after where feasible. Before deleting a test, keep any unique proof it held; a test that kills no sampled mutant is a candidate, not proof of waste: security, API, protocol, migration, storage, config-default and other independent contracts stay.

## Which check, when

Use fast local/unit/integration checks at the smallest adequate boundary during work. Use targeted UI checks for UI-specific bugs, interactions or layout. Where the work has connected journeys, run E2E when one first becomes available, after relevant risky cross-component changes and before product/candidate handoff to Chủ nhà (`sos gate ready`, which records the revision and result), not every worker return; reuse valid evidence rather than rerunning whole suites by slice count or agent stop. Assert persisted/output data as well as visible flow: a screenshot or open share sheet alone does not prove saved/exported content. For UI changes, inspect rendered output against agreed references, hierarchy, density, content and states. For documents, the artifact is what the reader receives (rendered page, PDF, published post), not the source file. Reuse existing leases for shared resources (simulators, devices, test databases) and resource limits.
