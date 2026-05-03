# Route Splice 04

## Canonical Thesis

A route list is ordered movement through constrained space.

This entry is a route-splice board built to make SolverForge list variables legible. The central object is not a generic graph; it is an ordered route with predecessor/successor links, insertion positions, downstream indices, and local route constraints.

## Product Meaning Map

- route plan = constrained space
- stations = list entries / visits
- numbered beads = route indices
- gold thread = ordered route
- moving arrowheads = successor links
- blue required station = required visit not yet in the route
- candidate arcs = insertion trials
- splice flash = one successor link replaced by two
- constraint badges = local `TIME`, `CAP`, `ORDER`, and `DIST` checks
- blocked link = compact detour through nearby route links
- goal glow = valid ordered route

## Runtime Plan

- total runtime: 116.0s
- fps: 60
- resolution: 1280x720
- default runtime: windowed `minifb` demo with live synthesized audio
- export runtime: explicit `--render`, raw BGR24 stdout, generated `target/demo_audio.wav`
- dependencies: `minifb`, `cpal`, `fundsp`

## Scene Timeline

| Scene | Time | Purpose |
|---|---:|---|
| Invocation | 0:00-0:10 | reveal the route plan |
| Stations | 0:10-0:22 | light route beads in list order |
| Successors | 0:22-0:34 | make predecessor/successor structure explicit |
| RequiredStation | 0:34-0:46 | introduce a required off-route station |
| InsertionSearch | 0:46-1:00 | score candidate insertion positions |
| Splice | 1:00-1:14 | break one link, form two, shift downstream indices |
| ConstraintRepair | 1:14-1:30 | pulse only local route constraints |
| Reroute | 1:30-1:40 | redraw a compact detour around a blocked link |
| FinalRoute | 1:40-1:48 | lock the completed route circuit |
| Scrolltext | 1:48-1:56 | land the list-variable thesis |

## Must-Have Rhetorical Beats

1. the viewer sees a numbered ordered route through stations
2. a required station appears off-route
3. candidate insertion arcs are evaluated
4. the chosen insertion visibly splices one link into two
5. downstream station indices shift
6. only local route constraints pulse
7. a later obstruction causes a compact detour
8. the final route is visibly valid and ordered

## Recommended Caption Script

- SOLVERFORGE PRESENTS
- ORDERED MOVEMENT THROUGH CONSTRAINED SPACE
- THE ROUTE IS A LIST
- EACH STATION HAS A SUCCESSOR
- A REQUIRED STATION APPEARS
- EVALUATE INSERTION POSITIONS
- TRY INSERT AT POS 4
- BREAK ONE LINK FORM TWO
- DOWNSTREAM INDICES SHIFT
- ONLY LOCAL CONSTRAINTS PULSE
- A LINK IS BLOCKED
- A COMPACT DETOUR REPAIRS THE ROUTE
- THE ORDERED PATH IS VALID
- LIST VARIABLES ARE ORDERED SPACE
- SOLVERFORGE

## Scrolltext Draft

LIST VARIABLES ARE ORDERED SPACE ... ONE REQUIRED STATION ENTERS THE ROUTE ... ONE SUCCESSOR LINK BREAKS ... TWO LINKS FORM ... DOWNSTREAM INDICES SHIFT ... LOCAL ROUTE CONSTRAINTS REPAIR ... SOLVERFORGE BERGAMO MMXXVI ...

## Implementation Doctrine

### Windowed demo first
The primary experience is `make run`: a real window with live audio. Headless export is still required, but it is an explicit render path for ffmpeg, not the default behavior.

### One route, one story
The route state should remain explicit: current order, candidate positions, selected insertion, affected downstream range, local constraints, and detour segment. Visual effects should always read as route/list operations.

### Song first
The score is a standalone arena-tracker synth song: low ostinato bass, gated chord stabs, punchy sampled-style drums, sparse lead hooks, echo returns, fills, pads, and a staged solo section. Visual scenes do not trigger musical events. The renderer may coexist with the song clock, but the music must stand as music before any route animation is considered.

### Top-down route plan over full 3D
Use a readable route plan and controlled depth cues. Prioritize route semantics over ornamental architecture.

### Renderer Priorities
1. readable route plan
2. numbered station beads
3. continuous gold route thread
4. successor arrows and labels
5. insertion candidate arcs and selected splice
6. downstream renumber ripple
7. local constraint badges
8. blocked-link detour and final glow

## Module Plan

```text
config.rs      constants and dimensions
audio/         standalone song engine plus live/deterministic WAV playback
math.rs        vec helpers and smoothstep
palette.rs     colors and blend helpers
framebuffer.rs raw BGR framebuffer and minifb conversion
scene.rs       semantic scene timing
route_plan.rs  station-backed route-plan layout
route.rs       list-variable route state
render.rs      software draw passes
main.rs        CLI orchestration
```

## Acceptance Warning

If a cold viewer cannot say "a required station was inserted into an ordered route" by the insertion scene, the demo has failed no matter how pretty it is.
