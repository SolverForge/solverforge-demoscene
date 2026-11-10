# ROUTING_10

SolverForge Routing for Hermes: a recorded trip-planning workflow, framed by
copper light, the animated Nous Girl and a subtitle dialog. 88 seconds,
1920×1080 at 60 fps.

## The video

The plugin is on screen from the first frame. There is no abstract routing
intro, synthetic trip counter or unrelated animation before the demonstration.

- **0–84 s:** a genuine Desktop conversation: request a Cortina day trip,
  review and approve the places, plan the round trip, remove a stop, choose
  Vinigo as the first visit, replan, then open the Apple Maps phone handoff.
  FilmCraft cuts the real recording and speeds up waits. Chat and map framing
  follow the action; the displayed results are the plugin's actual results.
- **84–88 s:** the SolverForge logo and repository address.

The Nous Girl speaks short, user-facing lines: where to ask, what to review,
how to change the trip and how to take it along. Her words type into the dialog
box. There are no full-frame subtitles or advisor header.

The rearranged soundtrack uses an eight-bar question-and-answer melody,
common-tone pad voicings and a shared eighth-note bass/drum pulse. Detuned
pads, gated snares and restrained tracker phrasing keep the series' synthwave
sound; the opening recedes beneath the voice and a dominant–tonic cadence closes.

## Run and render

```sh
make run       # live picture and audio; SPACE jumps to the next section
make render    # routing_10.mp4: frame and soundtrack, with black footage windows
make check
make clippy
cargo test --release
```

`routing-10-demo --live` exports the actual footage span plus `take`, `portrait`
and `dialog` rectangles as `x y width height`. The opening four seconds are
music over the same footage window, not a separate visual scene. The composer
reads this export rather than maintaining another timing/geometry table.

## Production

1. Record the real Desktop interaction with public demonstration locations,
   hiding unrelated sessions. Preserve real capture timestamps.
2. Import the master into FilmCraft. Cut and retime the request, review,
   confirmation, solve, changed request, replan and phone handoff. Save the
   editable project and an exact source/destination edit table.
3. Synthesize short narration against the edited action windows. Measure every
   line and refuse overlapping speech. One cue array drives voice timing,
   sprite gestures and typed dialog.
4. EffectCraft places the edited take inside its exported rectangle without
   changing its aspect ratio, animates the Nous Girl from the pet spritesheet,
   and types the words inside the dialog box. The soundtrack is already quiet
   during the speech section.
5. Verify the encoded result: duration, full decode, browser playback, actual
   request/review/replan/QR frames, complete wrapped dialog and closing frame.
   Signal checks do not establish that an arrangement sounds good.

Keep the editable production, cue files and dependent assets with the local
video delivery, outside disposable scratch. Do not put personal-route footage
or private session content in a repository or publish it as a demonstration.
