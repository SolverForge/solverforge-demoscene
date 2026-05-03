# ═══════════════════════════════════════════════════════════════════════════════
#   ██████╗ █████╗ ████████╗██╗  ██╗███████╗██████╗ ██████╗  █████╗ ██╗         
#  ██╔════╝██╔══██╗╚══██╔══╝██║  ██║██╔════╝██╔══██╗██╔══██╗██╔══██╗██║         
#  ██║     ███████║   ██║   ███████║█████╗  ██║  ██║██████╔╝███████║██║         
#  ██║     ██╔══██║   ██║   ██╔══██║██╔══╝  ██║  ██║██╔══██╗██╔══██║██║         
#  ╚██████╗██║  ██║   ██║   ██║  ██║███████╗██████╔╝██║  ██║██║  ██║███████╗    
#   ╚═════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝    
#
#  LIST VARIABLE ROUTE -- ROUTE SPLICE DEMO
#  WINDOWED RUST DEMO. LIVE AUDIO. EXPLICIT FFMPEG EXPORT.
# ═══════════════════════════════════════════════════════════════════════════════

.PHONY: help build run render render-gif clean check clippy loc

DEMO          := route-splice-04-demo
CARGO_FLAGS   := --release
RENDER_W      := 1280
RENDER_H      := 720
RENDER_FPS    := 60
RENDER_SECS   := 124
RENDER_OUT    := route_splice_04.mp4
RENDER_GIF    := route_splice_04.gif
AUDIO_OUT     := target/demo_audio.wav
FFMPEG_VCODEC := libx264
FFMPEG_CRF    := 18
FFMPEG_PRESET := slow

help: ## Show this help
	@echo ""
	@echo "  ROUTE_SPLICE_04 // LIST VARIABLE ROUTE"
	@echo ""
	@echo "  make build         Build release binary"
	@echo "  make run           Build and run the demo"
	@echo "  make render        Render MP4 with audio via ffmpeg"
	@echo "  make render-gif    Render GIF via ffmpeg"
	@echo "  make clean         Remove build artifacts + output"
	@echo "  make check         Run cargo check"
	@echo "  make clippy        Run clippy lints"
	@echo "  make loc           Count lines of Rust source"
	@echo ""
	@echo "  CONTROLS:  ESC = quit   SPACE = advance scene"
	@echo ""

build: ## Build release binary
	cargo build $(CARGO_FLAGS)

run: ## Build and run the demo in a window
	cargo run $(CARGO_FLAGS)

render: ## Render headless to MP4 via ffmpeg (with audio)
	@command -v ffmpeg >/dev/null 2>&1 || { echo "ERROR: ffmpeg not found. Install it first."; exit 1; }
	cargo run $(CARGO_FLAGS) -- --render $(RENDER_SECS) | \
		ffmpeg -y \
			-f rawvideo \
			-pixel_format bgr24 \
			-video_size $(RENDER_W)x$(RENDER_H) \
			-framerate $(RENDER_FPS) \
			-i - \
			-i $(AUDIO_OUT) \
			-c:v $(FFMPEG_VCODEC) \
			-c:a aac -b:a 192k \
			-r $(RENDER_FPS) \
			-crf $(FFMPEG_CRF) \
			-preset $(FFMPEG_PRESET) \
			-pix_fmt yuv420p \
			-movflags +faststart \
			-shortest \
			$(RENDER_OUT)
	@rm -f $(AUDIO_OUT)
	@ls -lh $(RENDER_OUT)

render-gif: ## Render headless to GIF (lower fps, palette-quantized)
	@command -v ffmpeg >/dev/null 2>&1 || { echo "ERROR: ffmpeg not found. Install it first."; exit 1; }
	cargo run $(CARGO_FLAGS) -- --render $(RENDER_SECS) | \
		ffmpeg -y \
			-f rawvideo \
			-pixel_format bgr24 \
			-video_size $(RENDER_W)x$(RENDER_H) \
			-framerate $(RENDER_FPS) \
			-i - \
			-vf "fps=15,scale=640:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer" \
			$(RENDER_GIF)
	@rm -f $(AUDIO_OUT)
	@ls -lh $(RENDER_GIF)

clean: ## Remove build artifacts and rendered output
	cargo clean
	@rm -f $(RENDER_OUT) $(RENDER_GIF) $(AUDIO_OUT)

check: ## Run cargo check
	cargo check $(CARGO_FLAGS)

clippy: ## Run clippy lints
	cargo clippy $(CARGO_FLAGS) -- -W clippy::all

loc: ## Count lines of Rust source
	@find src -name '*.rs' | sort | xargs wc -l
