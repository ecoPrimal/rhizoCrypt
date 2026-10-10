# rhizoCrypt — Evolution Record

The changelog is sorted by capability surface, not by date.
Each section tells the arc: what emerged, what it replaced, what it became.
The date-sorted history lives in `CHANGELOG_ARCHIVE.md`.

`H(data|epitope) < H(data|date) < H(data)`

---

## DAG Core — sessions, vertices, and append-only events

Tarpc-only RPC → dual-protocol JSON-RPC + tarpc with typed vertices, dual-format hash acceptance, and readiness gating.

- **Session lifecycle** (v0.13.0→v0.14.0): `dag.session.create/get/list/close` with enriched `SessionInfo` (agents, genesis, frontier); hex string + byte array hash acceptance (Gap 9) for loamSpine/sweetGrass interop.
- **Event append** (S67): `payload_ref` wiring fix — previously parsed but silently dropped; now stored on vertex via `VertexBuilder::with_payload()`; composition-critical for provenance trio.
- **Readiness gate** (S61): PG-60 — DAG methods return `-32002 NOT_READY` during startup instead of hanging; health/capabilities bypass gate.
- **Wire-name aliases** (S68): GAP-36 — 21 `provenance.*` methods map to `dag.*` canonical equivalents via `normalize_method()`; downstream springs call either name transparently.
- **Zero-copy hot path** (Wave 157k): `canonical_bytes` cache on `Vertex`, `BytesMut::freeze()` for CBOR, `resolved_id()` skips re-hashing on store reads.

---

## Dehydration & Merkle — sealing sessions into attestable roots

Session close → partial dehydrate, batch dehydration, and Merkle proof verification with loamSpine-compatible hex roots.

- **dag.dehydration.trigger** (S59→S69): Full dehydration pipeline producing 64-char hex Merkle root compatible with loamSpine `session_hash`; `dag.partial_dehydrate` for in-flight clone checkpoints (wetSpring sovereign pipeline).
- **Batch pipeline** (Wave 155n): `dehydrate_batch()` via JoinSet; `dag.dehydration.trigger_batch`; `append_vertices_batch()` single-lock batch append; `notify_dehydration_batch()` N→1 sweetGrass RPC.
- **dag.pipeline.ingest** (Wave 155n): Coordinated create + batch append + optional dehydrate in one RPC call.
- **SessionTreeHash** (Wave 143b): Content-addressable cache key wrapping `MerkleRoot` for duplicate detection; wired through full RPC stack + DashMap cache.
- **Gateway witnesses** (Wave 155b): `collect_gateway_witnesses()` enriches dehydration with cross-gate federation tier witnesses.

---

## Branch, Diff, Merge & Federate — version control over DAG composition

Linear append-only → git-like DAG operations for rootPulse signal graphs and cross-gate import.

- **4 DAG evolution methods** (Wave 60): `dag.branch` (fork at checkout), `dag.diff` (structural frontier diff), `dag.merge` (3-way merge vertex), `dag.federate` (remote import with duplicate skip).
- **Federate hardening** (Wave 155b): BTSP-DAG bridge — `ConnectionOrigin::BtspAuthenticated` auto-grants scopes; `verify_signatures` flag; `source_gate` metadata stamped on imported vertices; `FederateResponse.rejected` count.
- **Cross-gate provenance** (Wave 155b): Federate → sweetGrass `notify_provenance()` with `ProvenanceChain` and `WireWitnessRef` entries.
- **Handler optimization** (Wave 157k): Branch handlers call `impl_*` directly, bypassing tarpc self-clone; zero-copy federate via `into_iter()` on vertex arrays.

---

## Provenance Trio Integration — notifying sweetGrass and loamSpine

Isolated DAG store → wired notification pipeline feeding the provenance trio (rhizoCrypt + loamSpine + sweetGrass).

- **ProvenanceNotifier** (session 6→18): Dehydration summaries converted to wire format; silent `unwrap_or_default()` serialization → proper error propagation.
- **Composition tests** (S67): 8 tests validating skunkBat security event forwarding, full trio pipeline (SessionStart → DataCreate → SessionEnd → merkle.root), hex/URI payload_ref paths.
- **UDS provenance workflow** (S66): GAP-06 confirmed resolved — `test_uds_provenance_trio_dag_workflow` validates ludoSpring's exact call pattern over UDS.
- **Gossip injection** (Wave 157e): `GossipEmitter` at 3 lifecycle points (dehydrate, dehydrate_batch, federate) → swarmVine gossip relay; `gossip.spread` JSON-RPC method.

---

## Transport & Protocol — BTSP, riboCipher, dual-socket composition

TCP-primary → UDS-canonical with G65 negotiation, G66 transport abstraction, and C2 tarpc binary path.

- **UDS unconditional** (S23→S66): Unix socket canonical on all Unix platforms; socat-style verification documented; stale socket cleanup compliant since S23.
- **BTSP Phase 3** (S59): `btsp.negotiate` server handler — ChaCha20-Poly1305 encrypted framing; post-handshake flow redesign preserves plaintext backward compat.
- **BTSP ClientHello** (Wave 151b): Outbound 4-step handshake; `BtspUnixAdapter` for NDJSON over BTSP-authenticated UDS; fail-closed on strict mode.
- **G65 protocol negotiation** (Wave 156m): Single-socket `PROTOCOLS:` selection; `serve_tarpc_on_stream()` + `RpcClient::connect_negotiated()`.
- **G66 transport abstraction** (Wave 156s): `TransportListener`, `TransportEndpoint::platform_default()`, generic handshake on any `AsyncRead + AsyncWrite` stream; Windows-gnu cross-compile clean.
- **G64 C2 dual-socket** (Wave 156j): tarpc binary on `{primal}.tarpc.sock` alongside JSON-RPC; sub-ms intra-gate composition.
- **Mito-beacon signals** (Wave 116): riboCipher prefix handling evolved from exact match to full genetics signal family (`0xEC`/`0xED`/`0xEE`).

---

## Security & MethodGate — authorization before dispatch

Open localhost JSON-RPC → two-gate architecture with scope enforcement and BTSP-authenticated identity.

- **MethodGate** (S62): JH-0 adoption — Public/Protected classification; `-32001 PERMISSION_DENIED`; `RHIZOCRYPT_AUTH_MODE=enforced` activates enforcement.
- **Token verification** (S65): JH-1 framework — `TokenVerifier` trait, `_bearer_token` extraction, glob scope matching (`*`, `domain.*`, exact).
- **CapabilityVerifier** (Wave 118→128c): Evolved from `PresenceVerifier` to capability-discovered verifier with signing provider cache; fail-closed in Enforced mode when no crypto provider.
- **G63 SO_PEERCRED** (Wave 156e): Kernel-verified UID/GID/PID on every UDS connection; `auth.peer_info` includes peer credentials.
- **BTSP auth bridge** (Wave 155b): BTSP-authenticated callers auto-granted DAG scopes — family membership = DAG access.

---

## Cross-Gate Federation & Mesh — importing remote DAG state

Single-gate DAG → federated import with mesh event polling and signed trust vertices.

- **Mesh listener** (Wave 115→128c): Background poller for cross-gate trust events; error escalation after 3 consecutive failures; mesh poller signs trust event vertices via `SigningClient`.
- **Adapter-agnostic messaging** (Wave 120): Runtime strings evolved from "Songbird"/"bearDog" to "discovery service"/"signing provider"/"compute provider".
- **Vendor decoupling** (Wave 133e): `SongbirdClient` → `DiscoveryClient`; HTTP client types renamed to capability-neutral (`SigningHttpClient`, `StorageHttpClient`, etc.).
- **Dead vendor HTTP purge** (Wave 156b): 5 unused vendor modules removed (~4,100 lines, 132 dead tests) — zero production callers confirmed.

---

## rootPulse & Graph Executor — biomeOS trio activation

Standalone DAG methods → semantic aliases and step handlers wired into rootPulse commit graphs.

- **rootPulse handlers** (Wave 157k): `rootpulse.record_build` (harvest graph) + `rootpulse.dehydrate_state` (commit graph); semantic aliases `dag.append` → `dag.event.append`, `dehydrate` → `dag.dehydration.trigger`.
- **Registry alignment** (Wave 157a): 3-way self-audit — capability_registry vs METHOD_CATALOG vs handler dispatch; phantom `lifecycle.status` removed; `dag.session.tree_hash` added; 40 methods, zero phantoms.

---

## Neural API & Discovery — self-registration and capability symlinks

Manual socket paths → tiered discovery, `primal.announce`, and connect-probe liveness.

- **primal.announce** (Wave 43): Outbound startup registration to biomeOS Neural API — capabilities, 32+ methods, semantic mappings, cost hints, signal tier.
- **Connect-probe liveness** (Wave 119): `socket_is_alive()` replaces `Path::exists` — distinguishes live listeners from stale socket files.
- **Capability symlinks** (Wave 119): `dag.sock` → `rhizocrypt.sock` on bind (ecosystem standard v1.3.0).
- **Non-blocking announce** (Wave 49): Moved from blocking startup path to background `tokio::spawn()` — eliminates 7s UDS timeout when biomeOS unavailable.

---

## Storage & Persistence — redb-primary with sled deprecation

RocksDB/BearDog-coupled storage → pure Rust redb with capability-based adapter clients.

- **Storage backends** (v0.13.0): redb primary; sled deprecated then removed; `store_redb.rs` split with test extraction.
- **Integration trait wiring** (Wave 142b): `SigningClient`/`StorageClient`/`PermanentStorageClient` implement provider traits for dyn dispatch and mock injection.
- **AdapterFactory::from_transport** (Wave 142b): Structured dispatch eliminates Display→re-parse round-trip in all 5 capability clients.
- **Sovereignty hardening** (session 6): Cloud provider references removed from capability trait docs; agnostic "discovered at runtime" language.

---

## Architecture & Engineering — codebase evolution (cross-cutting)

Monolithic service with scattered debt → modular workspace with 92%+ coverage and enforced quality gates.

- **Structural refactoring** (Wave 149b→157k): `uds.rs` → module directory; `lib.rs` → 7 modules; `method_gate.rs` split; `transport.rs` → 6 submodules; zero production files >800L.
- **Deep debt discipline** (ongoing): Zero unsafe, zero async-trait in production, zero Box<dyn Error>, zero unwrap/expect in production, zero TODO/FIXME; all mocks cfg-gated.
- **G72 dep hygiene** (Wave 157g): wiremock removal cascades 46 crates eliminated (316→270 unique deps); `cargo deny` clean.
- **Coverage gate** (Wave 128): CI wired `cargo-llvm-cov --fail-under-lines 90`; 1,858 tests at 92.69% line coverage.
- **Cross-compile hygiene** (Wave 155i): Zero-warning builds for x86_64-linux, x86_64-windows-gnu, x86_64-linux-musl, aarch64-linux-musl.
- **Showcase fossilization** (Wave 49): 72 demo scripts archived; plasmidBin depot pattern replaces `target/release/` references.

---

*Full date-sorted history: `CHANGELOG_ARCHIVE.md`*
*Last compressed: Wave 172*
