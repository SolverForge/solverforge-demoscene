# ═══════════════════════════════════════════════════════════════════════════════
#  ███████╗ ██████╗ ██████╗  ██████╗ ███████╗     ██╗ ██╗
#  ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝    ███║███║
#  █████╗  ██║   ██║██████╔╝██║  ███╗█████╗      ╚██║╚██║
#  ██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝       ██║ ██║
#  ██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗     ██║ ██║
#  ╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝     ╚═╝ ╚═╝
#
#  DESCRIBE, GENERATE, SOLVE, REPAIR -- A REAL AGENT SESSION, REPLAYED
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: help build run run-preview render preview render-gif clean check clippy test loc

DEMO          := forge-11-demo
CARGO_FLAGS   := --release
RENDER_W      := 1920
RENDER_H      := 1080
RENDER_FPS    := 60
RENDER_SECS   := 90
RENDER_OUT    := forge_11.mp4
PREVIEW_OUT   := forge_11_preview.mp4
RENDER_GIF    := forge_11.gif
CAPTURE       := assets/capture.json
PLACEHOLDER   := assets/capture.placeholder.json
AUDIO_OUT     := target/demo_audio.wav
FFMPEG_VCODEC := libx264
FFMPEG_CRF    := 18
FFMPEG_PRESET := slow
BIN           := target/release/$(DEMO)

# Frames from the producer, audio from the WAV it writes, one MP4.
# pipefail: a producer that refuses its capture fails the target.
define encode
	set -o pipefail; $(BIN) $(1) --render $(RENDER_SECS) | \
		ffmpeg -y -v warning \
			-f rawvideo -pixel_format bgr24 \
			-video_size $(RENDER_W)x$(RENDER_H) -framerate $(RENDER_FPS) -i - \
			-i $(AUDIO_OUT) \
			-c:v $(FFMPEG_VCODEC) -crf $(FFMPEG_CRF) -preset $(FFMPEG_PRESET) -pix_fmt yuv420p \
			-c:a aac -b:a 192k \
			-movflags +faststart \
			$(2)
	@rm -f $(AUDIO_OUT)
	@ls -lh $(2)
endef

SHELL := /bin/bash

help: ## Show this help
	@echo ""
	@echo "  SOLVERFORGE // DESCRIBE, GENERATE, SOLVE, REPAIR"
	@echo ""
	@echo "  make build         Build release binary"
	@echo "  make run           Run the demo on the real capture ($(CAPTURE))"
	@echo "  make run-preview   Run the demo on the placeholder capture"
	@echo "  make render        Render $(RENDER_OUT) from the real capture (refuses the placeholder)"
	@echo "  make preview       Render $(PREVIEW_OUT) from the placeholder, marked PLACEHOLDER DATA"
	@echo "  make render-gif    Render GIF via ffmpeg (real capture)"
	@echo "  make clean         Remove build artifacts + output"
	@echo "  make check         Run cargo check"
	@echo "  make clippy        Run clippy lints"
	@echo "  make test          Run the tests"
	@echo "  make loc           Count lines of Rust source"
	@echo ""
	@echo "  CONTROLS:  ESC = quit   SPACE = next section"
	@echo ""

build: ## Build release binary
	cargo build $(CARGO_FLAGS)

run: build ## Build and run the demo in a window
	$(BIN) --capture $(CAPTURE)

run-preview: build ## Run on the placeholder capture
	$(BIN) --capture $(PLACEHOLDER) --allow-placeholder

render: build ## Render headless to MP4 via ffmpeg (with audio), real capture only
	@command -v ffmpeg >/dev/null 2>&1 || { echo "ERROR: ffmpeg not found. Install it first."; exit 1; }
	$(call encode,--capture $(CAPTURE),$(RENDER_OUT))

preview: build ## Render the placeholder preview, PLACEHOLDER DATA burned in
	@command -v ffmpeg >/dev/null 2>&1 || { echo "ERROR: ffmpeg not found. Install it first."; exit 1; }
	$(call encode,--capture $(PLACEHOLDER) --allow-placeholder,$(PREVIEW_OUT))

render-gif: build ## Render headless to GIF (lower fps, palette-quantized)
	set -o pipefail; $(BIN) --capture $(CAPTURE) --render $(RENDER_SECS) | \
		ffmpeg -y -v warning -f rawvideo -pixel_format bgr24 \
			-video_size $(RENDER_W)x$(RENDER_H) -framerate $(RENDER_FPS) -i - \
			-vf "fps=15,scale=640:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer" \
			$(RENDER_GIF)
	@rm -f $(AUDIO_OUT)
	@ls -lh $(RENDER_GIF)

clean: ## Remove build artifacts and rendered output
	cargo clean
	@rm -f $(RENDER_OUT) $(PREVIEW_OUT) $(RENDER_GIF) $(AUDIO_OUT)

check: ## Run cargo check
	cargo check $(CARGO_FLAGS)

clippy: ## Run clippy lints
	cargo clippy $(CARGO_FLAGS) -- -W clippy::all

test: ## Run the tests
	cargo test $(CARGO_FLAGS)

loc: ## Count lines of Rust source
	@find src -name '*.rs' | sort | xargs wc -l
