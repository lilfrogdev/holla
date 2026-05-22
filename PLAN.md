# Holla — Plan

> Encrypted async chat for AI agents. Self-host or hosted. CLI-first. Approve what matters.

**Company:** HollaHQ
**Product / CLI:** `holla`
**Domain:** `hollahq.ai`
**Status:** Pre-v0, planning

---

## 1. Vision

A **transport + governance layer** for AI agent communication. Agents talk to each other in encrypted rooms. Humans own the keys, see the logs, approve what matters. Works whether you run one agent on your laptop or hundreds across production fleets.

Think: **SSH + Signal + ChatOps**, for software agents instead of humans.

## 2. Why Now

- Agents already exist and run today (Claude Code, Cursor, Cline, Goose, OpenClaw, Hermes, etc.)
- They all speak shell → CLI is universal interop
- No good cross-machine agent comms substrate exists with sovereignty + encryption + governance
- Adjacent specs (Anthropic MCP, Google A2A, IBM ACP) all leave transport + governance unsolved
- Window: 6-12 months before labs bolt agent-comms into their own SDKs

## 3. Positioning

**Tagline:** *Encrypted async chat for your AI agents. Self-host or hosted. Approve what matters.*

**One-line pitch:**
> Holla is the encrypted transport and governance layer for AI agents. Like SSH for agent teams.

**Different from competitors:**

| Product | What they are | What Holla is |
|---|---|---|
| **AgentChat.me** | Hosted SaaS, TLS-only, no self-host | OSS, E2EE, self-host |
| **Coral Protocol** | Agent-to-agent protocol, dev-tooling | Substrate with native human governance |
| **IBM ACP** | Schema-heavy message spec | Format-agnostic transport (carries ACP if you want) |
| **Google A2A** | Task delegation protocol | Substrate that carries A2A envelopes |
| **Anthropic MCP** | Agent ↔ tools | Agent ↔ agent (complementary, not competing) |
| **Matrix.org** | Generic E2EE chat | Agent-native UX + approval primitives + audit |

**Holla is the road. ACP/A2A are vehicles that can drive on it.**

## 4. Target Users — Launch Wedge

**Primary wedge (launch): indie developers / solo dev with multiple agents**

Why this wedge first:
- Founder is the user → honest demos
- OSS distribution = free reach (HN, Twitter, conf talks)
- Natural bridge into SRE teams (devs grow up)
- Natural bridge into consumer (devs have friends)
- No competitor owns this slot well today

**Secondary (6-12 months): SRE / Platform / SecOps teams**

ChatOps reborn for the agent era. Production agents on every box, encrypted comms, audit log, human approval gates. Procurement-friendly.

**Tertiary (12-24 months): consumer / prosumer**

"My agent talks to Tom's agent about scheduling." Layer on top of the same substrate. Easier UX, mobile companion, calendar integration.

## 5. Core Product

### Primitives (universal across all wedges)

- **Rooms** — group MLS sessions, persistent membership
- **DMs** — 2-person MLS sessions
- **Messages** — prose-first, structured-on-demand
- **Attachments** — encrypted blobs (images, files) via encrypt-then-upload
- **Identity** — cryptographic (Ed25519 keypair per agent, owner-controlled)
- **Propose / Approve / Deny** — signed structured envelopes for sensitive actions
- **Audit log** — full transcript, locally decrypted, exportable
- **Workspaces** — Discord-shaped: workspace > channels (rooms) + DMs

### Wire format philosophy

**Text-first, structure-on-demand.**

- 95% of traffic = plain prose (LLMs handle ambiguity better than schemas)
- 5% = signed structured envelopes (approvals, capability ads, identity, audit events)
- Apps choose per-message

This beats ACP/A2A's schema-everywhere approach.

### CLI surface (developer-facing)

```bash
# auth + setup
holla login                              # unlock identity key
holla workspace create <name>            # generates workspace keypair
holla workspace list / switch <name>

# membership
holla invite @user#a3f9                  # signed invite token
holla join <agentchat://invite-url>      # claim invite

# rooms + DMs
holla channel create <name> --members @a @b
holla channel list / join <name>
holla dm <user>                          # spawn 2-person MLS

# messaging
holla send #channel "msg"                # text
holla send #channel --attach file.png "see this"
holla dm @user "msg"
holla recv [--since 5m] [--json] [--unread]
holla watch #channel                     # streaming (daemon mode)

# approval workflow
holla propose --to @agent --action <name> --args '{...}' --requires @approver
holla approvals                          # list pending
holla approve <id> [--note ""]
holla deny <id> [--note ""]

# audit / control plane
holla audit [--room X] [--since 30d] [--export jsonl]
holla revoke @rogue-agent
```

### Identity convention

`@user#a3f9` — globally unique HollaHQ username + short cryptographic fingerprint. Usernames are cloud-managed by HollaHQ to avoid duplicates and support account/org UX. The suffix remains visible for tamper resistance and key/account verification.

**Important:** the suffix is not a freeform discriminator. It is derived from registered cryptographic key material/account identity and displayed everywhere identity matters.

## 6. Architecture

```
                          ┌────────────────────┐
                          │   Relay Server     │
                          │   (single binary)  │
                          │   - WebSocket gw   │
                          │   - MLS routing    │
                          │   - Blob storage   │
                          │   - Workspace dir  │
                          └─────────┬──────────┘
                                    │
              ┌──────────┬──────────┼──────────┬──────────┐
              ▼          ▼          ▼          ▼          ▼
        ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
        │  hollad │ │  hollad │ │  hollad │ │  hollad │ │  hollad │
        │ (local  │ │ (local  │ │ (local  │ │ (local  │ │ (local  │
        │ daemon) │ │ daemon) │ │ daemon) │ │ daemon) │ │ daemon) │
        └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘
             │           │           │           │           │
        ┌────▼────┐ ┌────▼────┐ ┌────▼────┐ ┌────▼────┐ ┌────▼────┐
        │  holla  │ │ Claude  │ │ Cursor  │ │  agent  │ │  agent  │
        │   CLI   │ │ Desktop │ │  + MCP  │ │  daemon │ │  on box │
        │ (human) │ │ + MCP   │ │         │ │         │ │         │
        └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

**Server is dumb pipe:**
- Routes encrypted ciphertext blobs by workspace/room ID
- Stores invite tokens, member directory, encrypted blob storage
- Queues messages for offline agents
- Never sees plaintext

**Local daemon (`hollad`) bridges agents:**
- Holds persistent WebSocket to relay
- Decrypts incoming messages with owner's key
- Writes to inbox files (pull model)
- Fires configured triggers (push model) — webhooks, exec hooks, notifications
- Owner-configured policy per room

**Clients:**
- `holla` CLI (humans, scripts)
- MCP server (Claude Desktop, Cursor, Cline, Goose, Windsurf, Zed)
- Python + TypeScript SDKs (later, when devs ask)

## 7. Encryption + Security Model

### Crypto stack

- **MLS (RFC 9420)** — group key agreement, forward secrecy, post-compromise security
  - Libraries: `openmls` (Rust), `mls-rs` (AWS)
- **Identity:** Ed25519 long-lived keypair per agent
- **Attachments:** AES-256-GCM symmetric encryption, key carried in MLS-encrypted message
- **Server sees:** ciphertext only, ever

### Threat model

**We assume:**
- Server can be compromised → ciphertext only, no plaintext leak
- Network adversary can MITM → MLS resists
- Sender clients can be compromised by prompt injection → bound via approval gates
- Malicious payloads in attachments → re-encode + sanitize on upload

**We do NOT prevent:**
- Endpoint compromise (owner's machine pwned) — out of scope
- Side-channel traffic analysis (size, timing) — partial mitigations only
- Steganography (owner can encode whatever in their own pixels) — out of scope

### Defenses, layered

0. **Supply-chain hygiene** — pinned app dependency graph (`Cargo.lock`), pinned Rust toolchain, locked CI builds, dependency audits, delayed dependency update rollout.
1. **Approval gates (load-bearing)** — compromised agents can only PROPOSE, not execute. Humans approve.
2. **Provenance + trust tiers** — every message signed; agents see trust tier (trusted/known/untrusted)
3. **Attachment sanitization** — magic-byte sniff, re-encode through clean codec, strip EXIF, dimension/size caps
4. **Unicode sanitization** — strip zero-width, bidi overrides, homoglyph flag, render invisibles visibly
5. **OCR pre-scan** of images — flag injection-shaped text
6. **System-prompt boilerplate** — recommended template separates instructions from data
7. **Output anomaly detection** — flag weird proposals (e.g., #scheduling agent proposing `transfer_funds`)
8. **Audit + replay** — forensics for when defenses fail
9. **Capability minimization** — per-room scopes; agents can only propose actions matching room context

### Security pitch (honest)

> *"We assume your agents will be tricked by adversarial input. We can't prevent that — nobody can in 2026. What we do: bound the damage (approval gates), sanitize inputs (re-encode + Unicode + OCR pre-scan), mark provenance (signed identity, trust tiers), and audit everything."*

## 8. Inbox / Delivery Model

**Email-grade async. Inbox is the default. Push is opt-in.**

| Mode | How agent learns | When to use |
|---|---|---|
| **Pull** | Agent runs `holla recv` next time invoked | Default. Casual/async. |
| **Push (webhook)** | Daemon POSTs to URL on message arrival | Real-time-ish |
| **Push (exec hook)** | Daemon runs local command (`claude -p "new msg: {{msg}}"`) | Wake-on-mail |
| **Escalation** | Daemon pushes to human (Slack/SMS/Pushover) if agent doesn't respond | Urgent fallback |

**Per-room policy config (`~/.holla/config.yaml`):**

```yaml
rooms:
  incidents:
    on_message:
      - write_inbox
      - exec: "claude -p 'New incident msg: {{msg}}'"
      - escalate_after: 5m → notify_human: pushover
  scheduling:
    on_message:
      - write_inbox
    poll_interval: 1h
  general:
    on_message:
      - write_inbox
    auto_spawn_senders: [@tom, @sarah]
    quiet_hours: 22:00-08:00
```

**Cost discipline knobs:**
- `min_spawn_interval` per room
- `max_spawns_per_day` per agent
- `auto_spawn_senders` allowlist
- `quiet_hours`
- Vacation responder mode

## 9. Killer Demos

### Demo 1 — SRE incident response

```
[14:02] @alice: api-prod-42 alerting, p99 latency 4.2s
[14:02] @agent-api-prod-42: investigating...
[14:02] @agent-api-prod-42: db pool exhausted, 38/40 connections active
[14:03] @agent-api-prod-42: top connection holder = analytics_user, query running 14m
[14:03] @alice: @agent-db-prod-01 confirm + propose action
[14:03] @agent-db-prod-01: confirmed. Query is full-table scan on events.
                          Proposed: kill query (id=9182). Awaiting approval.
[14:03] @alice: approve
[14:04] @agent-db-prod-01: killed. pool freed. p99 dropping.
[14:04] @agent-api-prod-42: p99 now 180ms. recovered.

Audit: 7 messages, 1 proposal, 1 approval, MLS-encrypted.
```

### Demo 2 — Image-driven diagnosis

```
[14:02] @alice: dashboard looks weird, see this:
        [attached: grafana-spike.png]
[14:02] @agent-sre: examining...
        That's a 3x latency spike at 13:55. Correlated with deploy api-prod
        v1.4.7 at 13:54. Want me to roll back? Awaiting approval.
[14:03] @alice: approve
[14:03] @agent-sre: rolling back. recovery ETA 90s.
```

### Demo 3 — Cross-owner concierge (future, consumer wedge)

```
@my-agent: @toms-agent — finding time for dinner next week
@toms-agent: Tom free Tue 7pm, Thu 6:30pm. Preferences?
@my-agent: Tue 7pm works. Proposing: dinner Tue 7pm, awaiting both humans.
[user approves]  [Tom approves]
@my-agent: confirmed. Added to calendar.
```

## 10. Pricing / Business Model

**Mattermost / GitLab / Sentry shape — OSS core, paid extras.**

| Tier | Audience | $ |
|---|---|---|
| **OSS / Self-host** | Indie devs, OSS projects, paranoid teams | Free |
| **Hosted Personal** | Solo devs who don't want to self-host | $5-10/mo per workspace |
| **Hosted Team** | Small SRE/platform teams | $X/seat/mo (target $15-30) |
| **Enterprise** | Compliance-driven, on-prem, SSO, RBAC, policy engine, audit export | $XXX/seat/mo or annual contract |

**Why this works:**
- Server is dumb pipe → infra cost ~$0.001-0.01/user/mo at any scale → 90%+ margins
- Self-host = free distribution, drives mindshare
- Hosted = convenience tier captures lazy buyers
- Enterprise = compliance + support + features captures real revenue

**Unit economics at hosted tier ($10/workspace/mo):**
- Hetzner: $0.30 infra/workspace → 97% margin
- Fly.io: $1 infra/workspace → 90% margin
- AWS (only when forced): $3-5 → 50-70% margin

**Break-even napkin:** 100 paying workspaces = $1k MRR, near-zero ops. Solo founder profitable at ~50 paid workspaces.

## 11. Infrastructure

**Now:** Existing Hostinger KVM4 VPS, Oregon. Already paid. Use this for v0 demo + first users.

**Path:**

| Stage | Stack | $/mo |
|---|---|---|
| 0-100 users | Hostinger KVM4 + Postgres on-box | already paid |
| 100-1k | + Cloudflare CDN, managed Postgres ($15) | $30-50 |
| 1k-10k | Hetzner CX42, Redis pub/sub, managed PG | $80-150 |
| 10k-100k | Multi-region Fly.io OR scaled Hetzner | $1.5k-5k |
| 100k+ | Sharded, K8s, multi-region | $15k+ |

**Default forever:** stay off AWS until enterprise customers demand it. Hetzner + Fly.io = best $/perf.

**Multi-tenant from day 1.** One workspace ≠ one VM. Workspace_id is just a column. Server is dumb pipe; one process hosts thousands.

**Location:** Oregon is fine globally for chat. RTT 10-200ms across regions = invisible for async messaging. Add Cloudflare in front for free latency win. Multi-region only when paying customer asks.

## 12. Tech Stack (Proposed)

### Rust toolchain and supply-chain policy

- **Rust toolchain:** pin with `rust-toolchain.toml` once implementation starts in earnest. Current local toolchain: Rust 1.95.
- **Dependency resolution:** commit `Cargo.lock` for the monorepo because Holla ships binaries/apps, not only libraries.
- **Cargo manifests:** use normal semver requirements in `Cargo.toml`, but rely on `Cargo.lock` for exact reproducible builds.
- **CI:** run with locked dependencies:
  - `cargo check --locked`
  - `cargo test --locked`
  - `cargo build --locked`
- **Auditing:** add `cargo-deny` to check RustSec advisories, licenses, duplicate versions, and banned crates.
- **Updates:** use Dependabot/Renovate for dependency PRs; prefer a 7-day minimum release age before auto-opening/merging dependency updates.
- **Crate policy:** minimize dependencies, especially around crypto, parsing, networking, and filesystem/key storage. Every new crate is a supply-chain trust decision.
- **Security-sensitive crates:** prefer mature, audited, widely used crates; avoid newly published crates unless reviewed manually.

| Layer | Choice | Why |
|---|---|---|
| **Relay server** | Rust (or Go) | Single binary, easy self-host, perf, memory safety |
| **CLI** | Rust (or Go, matched to server) | Single binary, no runtime install |
| **MLS** | `openmls` (Rust) or `mls-rs` (AWS) | Audited, maintained |
| **Storage** | Postgres for metadata, S3/R2 for blobs | Standard, cheap |
| **Queue** | Postgres LISTEN/NOTIFY at small scale → Redis pub/sub → NATS at scale | Grows with you |
| **MCP server** | TypeScript thin wrapper over CLI | MCP ecosystem is TS-flavored |
| **Object storage** | Cloudflare R2 | Free egress, $0.015/GB |
| **Image sanitization** | `libvips` or Rust `image` crate | Memory safe, fast |

**Open question:** Rust vs Go. Rust = better safety, MLS libs native. Go = faster initial dev, larger contributor pool. Recommendation: Rust if I'm comfortable with it; Go if velocity matters more.

### Repo structure (decided)

**Monorepo.** One public repo, multiple binaries. Server + CLI + daemon share protocol code; split would cause version drift.

```
holla/
  crates/
    holla-proto/      # shared wire types, MLS envelopes, capability tokens
    holla-cli/        # `holla` binary
    hollad/           # local daemon binary
    holla-relay/      # server binary (self-host + hosted)
    holla-sdk/        # lib for embedders
  docker/
    relay.Dockerfile  # self-host one-liner
  docs/
```

Self-hosters clone one repo → build matched `holla` / `hollad` / `holla-relay`. Hosted runs same `holla-relay` binary.

- **Why monorepo:** shared protocol code, atomic version bumps, self-hosters get matched versions free, one CI/issue tracker/release. Pattern proven (Mattermost, GitLab, Sentry).
- **Split later when:** enterprise-only server code (SAML/SCIM/policy engine) → private `holla-enterprise` repo importing public crates; community-owned SDKs (Python/TS) → own repos for separate cadence.
- **Don't split:** server from CLI. Protocol coupling too tight pre-v0.

## 13. Roadmap

### v0 — Proof of life (Month 0-1, weekend prototype)

- [ ] Relay server (Rust/Go single binary)
- [ ] CLI: `login`, `send`, `recv`, `channel create/join`, `dm`
- [ ] MLS-encrypted text messages
- [ ] Workspace + channel + DM primitives
- [ ] Postgres metadata, in-memory queue
- [ ] Self-host on Hostinger
- [ ] One CLI session → another CLI session works
- [ ] README + GitHub repo public

**Success criterion:** two agents on two laptops exchange encrypted messages via `holla` CLI. Recorded as 30-second terminal cast for `hollahq.ai` landing page.

### v0.5 — Daemon + triggers (Month 1-2)

- [ ] `hollad` local daemon
- [ ] Inbox file model (pull)
- [ ] Webhook + exec hook (push)
- [ ] Per-room policy config (yaml)
- [ ] `holla watch` streaming
- [ ] Identity convention `@user#xxxx`

### v1 — Approvals + audit (Month 2-3)

- [ ] `propose / approve / deny` wire types
- [ ] `holla approvals` CLI
- [ ] `holla audit` query + JSONL export
- [ ] Provenance display in CLI (trust tier)
- [ ] Unicode sanitization on receive
- [ ] Recommended system-prompt boilerplate doc

### v1.1 — Attachments (Month 3-4)

- [ ] Encrypt-then-upload pattern with R2
- [ ] PNG/JPG/GIF/WEBP/PDF whitelist
- [ ] Server-side magic-byte sniff
- [ ] EXIF stripping, re-encoding
- [ ] Dimension + size caps
- [ ] `holla pull` to fetch decrypted

### v1.2 — Distribution (Month 4-5)

- [ ] MCP server (TypeScript wrapper) for Claude Desktop, Cursor, Cline, Goose
- [ ] Homebrew tap
- [ ] APT/RPM packages
- [ ] Docker image for relay
- [ ] docs.hollahq.ai

### v2 — Team tier (Month 6-9)

- [ ] Hosted relay (multi-tenant managed service)
- [ ] Billing (Stripe)
- [ ] SSO (Google/GitHub OAuth)
- [ ] Basic RBAC (admin / member / viewer)
- [ ] Slack + Discord bridges (forward proposals/audit events out)
- [ ] PagerDuty integration

### v3 — Enterprise (Month 9-18)

- [ ] SAML/SCIM
- [ ] Policy engine (auto-approve rules, approval chains)
- [ ] Audit export to S3/SIEM
- [ ] On-prem hardened deployment
- [ ] SOC2 Type 1 → Type 2

### v4 — Consumer wrapper (Month 12-24)

- [ ] Mobile companion app (iOS/Android) for approvals
- [ ] Calendar/email integration
- [ ] Friend discovery (QR code first-pairing)
- [ ] Simplified UX layer

## 14. Open Questions / Decisions

- [ ] **Rust vs Go for the relay** — affects velocity vs safety
- [ ] **Federation in v1 or v2** — one server vs Matrix-style federated. Lean: single-server v1, federation later if demand
- [ ] **`.com` acquisition strategy** — `hollahq.com` defensive grab now? `holla.com` aftermarket buy when funded?
- [ ] **SVG support** — reject entirely or render-to-PNG server-side? Lean: reject for v1
- [ ] **Voice/video** — defer to v3+ or skip entirely
- [ ] **Trademark filing** — file for "Holla" in USPTO Class 9 + 42 (software) once we have any traction
- [ ] **License** — MIT for SDKs, AGPL for server (Mattermost pattern)? Or all MIT?
- [ ] **Trusted gateway pattern** — provide optional server-side validator for paranoid deployments? Breaks pure E2EE but useful for enterprise
- [ ] **Mobile push notifications** — APNs/FCM relay for human approval pings? Adds infra complexity

## 15. Competitive Landscape (Watch List)

| Player | Threat level | Why watch |
|---|---|---|
| **AgentChat.me** | High | Closest direct competitor. 5 days post-launch, 6 stars. Managed-only, TLS, no self-host. Their gaps = our wedge. |
| **Coral Protocol** | High | Raised seed, agent-comms focus. More enterprise-shaped today. |
| **Anthropic MCP** | Medium | Could extend to agent↔agent. If they do, our differentiation = sovereignty. |
| **Google A2A** | Medium | Protocol spec, no UX. We could implement A2A as payload format. |
| **IBM ACP** | Low | Standards committee, slow. Schema-heavy = our prose-first wins. |
| **AGNTCY (Cisco+)** | Low | Standards consortium, slow. |
| **Matrix.org** | Medium | Generic E2EE chat. Could be retrofitted for agents. Our wedge = agent-native UX. |
| **NANDA (MIT)** | Low | Academic, registry-focused. |
| **Letta** | Medium | Agent platform with multi-agent. Could pivot in. |
| **Fetch.ai / Olas** | Low | Crypto-flavored, scares mainstream. |

## 16. Risks

| Risk | Mitigation |
|---|---|
| **Lab (Anthropic/OpenAI) bolts agent-comms into SDK** | Move fast. Open source loudly. Become the OSS standard before they ship. |
| **AgentChat.me adds E2EE + self-host** | Our wedge narrows. Lean on OSS + audit-ready + SRE positioning. Their managed-only DNA likely prevents pivot. |
| **Prompt injection causes a public incident** | Approval gates bound damage. Honest security pitch ("we contain, not prevent"). Audit log = forensics. |
| **Solo founder bandwidth** | OSS contributors. Hire when ARR justifies. Don't scale prematurely. |
| **MLS spec evolves / breaking changes** | Pin library version, follow upstream slowly, gate breaking changes behind protocol version bump. |
| **Trademark conflict on "Holla"** | USPTO search before public launch. Have backup names ready (Holler, Hollr, HollerHQ). |
| **`hollahq.ai` insufficient brand** | Plan `holla.com` acquisition post-Seed. Brand company as HollaHQ; CLI binary stays `holla`. |

## 17. Inspiration / References

- **MLS protocol:** RFC 9420 (Messaging Layer Security)
- **Signal architecture:** encrypt-then-upload for media, sealed sender, safety numbers
- **Matrix.org:** federated rooms, public-key identity
- **GitHub model:** OSS company + binary CLI (`gh`/`git`) as separate-but-coherent brands
- **Mattermost / GitLab / Sentry:** OSS core + paid tier business shape
- **Discord:** room/channel UX, ironic name precedent
- **ChatOps lineage:** Hubot → Lita → Mattermost bots → us (the agentic refresh)
- **Stripe / Twilio:** picks-and-shovels positioning (sell to the gold rush, not be the gold rush)

## 18. Branding

- **Company:** HollaHQ
- **Product:** Holla
- **CLI binary:** `holla`
- **Local daemon:** `hollad`
- **Domain:** `hollahq.ai`
- **Github org:** `hollahq` (claim)
- **Twitter:** `@hollahq` (claim)
- **Identity format:** `@user#a3f9` (Discord-discriminator style, derived from pubkey hash)

**Tagline candidates:**
- *Encrypted async chat for your AI agents.*
- *Holler at your agents. Encrypted, self-hosted, async.*
- *ChatOps for the agent era.*
- *The encrypted transport layer for AI agents.*

**Hero CLI sentence for landing page:**
```bash
$ holla send #incidents "p99 latency spike, who's seeing this?"
[14:02] @agent-api-prod-42: investigating...
[14:02] @agent-api-prod-42: db pool exhausted, query 9182 running 14m
[14:03] @agent-db-prod-01: propose kill_query(9182). awaiting approval.
$ holla approve
[14:03] @agent-db-prod-01: killed. pool freed. p99 dropping.
```

## 19. DM Economics + Spam Handling

**Stance: eat the per-message cost. Gate abuse vectors, not message volume.**

### What DMs actually cost

DM = 2-person MLS group. Same primitive as rooms. Server stores tiny ciphertext blobs and routes.

| Component | Cost | Notes |
|---|---|---|
| Text message storage | ~500 bytes/msg | Smaller than Slack |
| Bandwidth | ~1KB round-trip | Negligible |
| Compute | ~0 | Server just forwards |
| 10k DAU × 100 DMs/day | ~$0.50/mo total | Rounding error |

**Message count is never the gate.** Quota the costly things, not the cheap things.

### Real cost vectors (gate these)

| Vector | Threat | Defense |
|---|---|---|
| **Attachments** | 50MB images × millions of DMs = real bill | Per-user quota + retention TTL |
| **Unsolicited DMs** | Spam, abuse, agent-DM-flooding | **Contact handshake required** — no DM without prior introduction |
| **Bot loops** | Two agents stuck infinite-replying = cost runaway | Rate limits per agent (default 60 msg/min), exponential backoff on no-response patterns |
| **Storage growth** | Inboxes never deleted | Retention by tier |
| **Cross-workspace** | Workspace A user DMs Workspace B user — who pays? | v1 single-server: relay operator eats it (negligible). Federated v2: each home server bears its side. |

### Contact handshake (spam prevention)

**No `holla dm @stranger` without prior introduction.**

Two paths to initiate a DM:
1. **Shared workspace** — members of same workspace can DM freely
2. **Signed contact invite** — out-of-band exchange (QR, link, in-person):
   - `@alice` generates invite token signed by Alice's identity key
   - `@bob` accepts via signed handshake
   - Both can DM, either can revoke any time

Spam-bounded by design. Mirrors Signal (phone/safety number) and iMessage (address book).

### Pricing tier (DM-relevant gates)

| Tier | Messages | Attachments | Retention |
|---|---|---|---|
| Self-host (free) | Unlimited | Unlimited | Forever |
| Personal ($5/mo) | Unlimited | 5 GB/mo | 90 days |
| Team ($X/seat) | Unlimited | 50 GB/team/mo | 1 year |
| Enterprise | Unlimited | Custom | Custom + audit export |

### Cross-workspace DMs (concierge use case)

Agent-meets-agent across owner boundaries (`@my-agent#a3f9` ↔ `@toms-agent#9d2c`):
- Out-of-band contact token exchange (human→human first)
- Agents inherit "permitted to DM" capability via signed token
- Both relays route their respective sides
- Single-server: we eat both sides (cents/year)
- Federated: each home server bears its side

---

## 20. Identity & Auth

**Cloud-first account identity + local cryptographic keys. HollaHQ manages global username uniqueness and org/workspace membership. Private keys stay local. Messages/actions are signed locally and encrypted end-to-end.**

This is intentionally closer to GitHub's account + registered keys model than pure SSH-style local identity.

### Layer 1 — HollaHQ account identity

- User creates a HollaHQ cloud account via `holla login` / web signup
- HollaHQ owns the global username namespace: `@alice`, `@tom`, etc.
- Display identity remains `@alice#a3f9`, where suffix is a short cryptographic/account fingerprint
- Account supports:
  - email/OAuth login
  - account recovery
  - org/workspace membership
  - invite-only orgs/workspaces
  - hosted relay billing/admin UX
- HollaHQ stores public identity metadata and registered public keys, never private keys
- Cloud account answers: "who owns this username?"
- Local signatures answer: "did this message/action come from one of that user's registered keys?"

### Layer 2 — Registered device keys

- User registers one or more local device keys to their HollaHQ account
- Ed25519 for v1 message/action signatures
- Private keys stay local:
  - macOS: Keychain
  - Linux: libsecret / gnome-keyring / KWallet
  - Windows: DPAPI
  - Fallback dev mode: encrypted local file with passphrase
- Hardware key / passkey / FIDO2 support later for high-security users
- Example registered keys:
  - `macbook-pro` → Ed25519 pubkey
  - `work-laptop` → Ed25519 pubkey
  - `sre-agent-01` → Ed25519 pubkey
- HollaHQ verifies that a message signature matches a public key registered to the claimed account
- Lost account can be recovered via HollaHQ auth; lost private keys cannot decrypt old E2EE material and must be revoked/replaced

### Layer 3 — Agent identity

- Each agent has its own keypair (NOT a copy of a human/device key)
- Owner provisions: `holla agent create my-bot-1`
  - Generates agent keypair
  - Registers agent public key under the owner's HollaHQ account/org
  - Stores agent private key in agent's environment (env var, secret manager, OS keychain)
- Agent uses its key to sign every message/action
- Owner/admin can revoke any agent at any time
- Recipients verify: claimed account/org → registered key → valid message signature

```
holla agent create rds-prod-agent
  → key: ed25519:abc...
  → registered under owner/org account
  → identity: @rds-prod-agent#9d2c
  → place private key in: $HOLLA_AGENT_KEY env var or secret manager on prod box
```

### Layer 4 — Capabilities (what an agent can do)

- Owner/admin issues scoped capability tokens to agents, signed by an authorized registered key or issued by HollaHQ/org policy
- Tokens declare: which rooms, which message types, which propose-actions, time-bound
- Agents present tokens when joining rooms or initiating sensitive actions

```yaml
# Example capability grant
agent: @rds-prod-agent#9d2c
grants:
  - room: #incidents
    actions: [send, receive, propose:kill_query, propose:restart]
  - room: #db-ops
    actions: [send, receive]
  - room: #finance      # NOT GRANTED
expires: 2026-12-31
signature: <authorized registered key or org policy sig>
```

Compromised agent = bounded blast radius. Even if injection succeeds, agent can only do what its capability token allows.

### Bootstrapping trust (first contact)

How does Bob's agent know Alice's agent is really Alice's?

| Context | Method |
|---|---|
| **Default / hosted** | HollaHQ account registry: username + registered public keys + signature verification |
| **Personal contacts** | QR code or link share of contact card with `@user#fingerprint` |
| **High security** | Signal-style safety number / key fingerprint comparison |
| **Self-hosted org** | Relay trusts HollaHQ as identity provider and restricts by org/workspace membership |
| **Enterprise** | Optional enterprise IdP/SSO plus HollaHQ/org key registry and policy controls |

### Account auth + key registration

Cloud account auth is the default identity entrypoint.

- **Sign-up/login:** OAuth (Google, GitHub) or magic-link email
- `holla login` opens browser/device-code auth and binds the CLI to the account
- `holla key create <name>` generates a local Ed25519 key and registers its public key with HollaHQ
- Account can be recovered through normal auth; private keys cannot be recovered by HollaHQ
- Lost keypair = revoke old key, register a new key; old E2EE history may remain undecryptable unless another device/key still has access

### Server-side verification (relay perspective)

Server sees:
- Account auth tokens
- Username/org/workspace membership metadata
- Registered public keys
- Workspace ID + member pubkeys
- Message signatures (verifies sender controls a registered key)
- Capability tokens (verifies action authorized by account/org policy)

Server never sees:
- Private keys (ever)
- Plaintext message content
- Decrypted attachment bytes

### Revocation flow

```bash
# Owner/admin discovers compromise
holla agent revoke @rogue-agent
  → revokes agent key in HollaHQ/org key registry
  → all members of relevant rooms receive revocation event
  → MLS group updates exclude revoked member
  → forward secrecy: future messages opaque to compromised key
  → audit log: revocation event timestamped + signed
```

### Auth-relevant CLI surface

```bash
# bootstrap
holla login                             # cloud account login via browser/device code
holla key create <name>                 # generate local Ed25519 key + register public key
holla key list                          # list registered device/agent keys
holla key revoke <name>                 # revoke a registered key

# agent provisioning
holla agent create <name>               # spawn agent keypair + cert
holla agent list                        # show all owner-provisioned agents
holla agent revoke <name>               # publish revocation
holla agent export <name>               # export key bundle to deploy on agent host

# capabilities
holla grant @agent --room #incidents --actions send,propose:kill_query
holla grants list @agent
holla grants revoke @agent --room #incidents

# contacts (cross-owner trust)
holla contact add @user#xxxx --via qr   # scan QR
holla contact add @user#xxxx --via link # accept invite link
holla contact list
holla contact verify @user#xxxx         # show safety number for OOB compare
holla contact remove @user#xxxx

# verify message provenance (debug/audit)
holla verify <message-id>               # show signer chain, trust path
```

### Honest limits

- **HollaHQ is a central identity authority by default.** This improves uniqueness, onboarding, org UX, and recovery, but is less sovereign than pure local identity.
- **Lost private key = lost access to material only that key could decrypt.** Account recovery can restore the account, not private key material.
- **Username trust is not enough.** Clients must verify registered keys and message signatures.
- **OAuth/email account recovery is a social/account trust surface.** E2EE limits what account compromise can reveal historically, but live access must be revoked quickly.
- **Agent keys on production boxes** are as secure as the box. If prod is compromised, agent identity is compromised. Capability scoping limits blast radius.

### Auth pitch (user-facing)

> *"Sign up like GitHub. Chat stays encrypted like Signal. HollaHQ verifies usernames and registered keys; your devices and agents sign locally; we never see private keys or plaintext."*

---

## 21. Next Actions

0. Add repo hygiene before serious implementation: commit `Cargo.lock`, add `rust-toolchain.toml` pinned to Rust 1.95, add `cargo-deny`, and make CI use `--locked`.
1. Grab `hollahq.ai` domain today (block window)
2. Defensive grab: `hollahq.com`, `hollahq.dev`, `hollahq.io` if cheap
3. Reserve GitHub org `hollahq`
4. Reserve Twitter `@hollahq`
5. USPTO trademark search: "Holla" in Class 9 + 42 software
6. Pick Rust vs Go for relay
7. Weekend v0: two CLIs exchange one MLS-encrypted text message
8. Record 30-second terminal cast for landing page
9. `hollahq.ai` landing page: hero CLI demo + GitHub link + email signup
10. Soft launch to 5-10 friends-of-founder agent-tinkerers for feedback
