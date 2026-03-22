# ═══════════════════════════════════════════════════════════════════════════
#  ███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
#  ██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
#  ███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗
#  ╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
#  ███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
#  ╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
#
#  SCREENSAVER_03 -- SWAY PHOSPHOR RITUAL
#  SILENT MODE. EMERALD GLYPH RAIN. SWAY AFTER DARK.
#
#  CODED IN RUST. NO JAVASCRIPT WAS HARMED.
# ═══════════════════════════════════════════════════════════════════════════

.PHONY: help build run render render-gif clean check clippy loc

# ── Configuration ─────────────────────────────────────────────────────────
DEMO            := solverforge-screensaver
CARGO_FLAGS     := --release

# Render settings
RENDER_W        := 1920
RENDER_H        := 1080
RENDER_FPS      := 60
RENDER_SECS     := 30
RENDER_OUT      := screensaver_03.mp4
RENDER_GIF      := screensaver_03.gif

# ffmpeg encoding settings
FFMPEG_VCODEC   := libx264
FFMPEG_CRF      := 18
FFMPEG_PRESET   := slow

# ── Default target ────────────────────────────────────────────────────────
help: ## Show this help
	@echo ""
	@echo " ███████╗ ██████╗ ██╗    ██╗   ██╗███████╗██████╗ ███████╗ ██████╗ ██████╗  ██████╗ ███████╗"
	@echo " ██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝"
	@echo " ███████╗██║   ██║██║    ██║   ██║█████╗  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗█████╗"
	@echo " ╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝"
	@echo " ███████║╚██████╔╝███████╗╚████╔╝ ███████╗██║  ██║██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗"
	@echo " ╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝"
	@echo "            ── SCREENSAVER_03 // SWAY PHOSPHOR RITUAL ──"
	@echo ""
	@echo " SOLVERFORGE // SWAY ONLY // EMERALD GLYPH RAIN // CRT THREAT DISPLAY."
	@echo ""
	@echo "  ┌─────────────────────────────────────────────────────┐"
	@echo "  │  TARGETS                                            │"
	@echo "  ├─────────────────────────────────────────────────────┤"
	@echo "  │  make build         Build release binary            │"
	@echo "  │  make run           Build and run the sway saver    │"
	@echo "  │  make render        Render to MP4 via ffmpeg        │"
	@echo "  │  make render-gif    Render to GIF (shareable)       │"
	@echo "  │  make clean         Remove build artifacts + output │"
	@echo "  │  make check         Run cargo check                 │"
	@echo "  │  make clippy        Run clippy lints                │"
	@echo "  │  make loc           Count lines of code             │"
	@echo "  └─────────────────────────────────────────────────────┘"
	@echo ""
	@echo "  CONTROLS:  ESC = quit   SPACE = toggle overlay"
	@echo ""

# ── Build ─────────────────────────────────────────────────────────────────
build: ## Build release binary
	@echo "═══ COMPILING $(DEMO) (release) ═══"
	cargo build $(CARGO_FLAGS)
	@echo "═══ DONE ═══"

# ── Run ───────────────────────────────────────────────────────────────────
run: ## Build and run the screensaver in a window
	@echo "═══ LAUNCHING $(DEMO) ═══"
	@echo "    ESC = quit // SPACE = toggle overlay"
	@echo ""
	cargo run $(CARGO_FLAGS)

# ── Render to MP4 ────────────────────────────────────────────────────────
render: ## Render headless to MP4 via ffmpeg
	@echo "═══ RENDERING $(RENDER_SECS)s @ $(RENDER_W)x$(RENDER_H) $(RENDER_FPS)fps ═══"
	@echo "    Output: $(RENDER_OUT)"
	@echo ""
	@command -v ffmpeg >/dev/null 2>&1 || { echo "ERROR: ffmpeg not found. Install it first."; exit 1; }
	cargo run $(CARGO_FLAGS) -- --render $(RENDER_SECS) | \
		ffmpeg -y \
			-f rawvideo \
			-pixel_format bgr24 \
			-video_size $(RENDER_W)x$(RENDER_H) \
			-framerate $(RENDER_FPS) \
			-i - \
			-c:v $(FFMPEG_VCODEC) \
			-r $(RENDER_FPS) \
			-crf $(FFMPEG_CRF) \
			-preset $(FFMPEG_PRESET) \
			-pix_fmt yuv420p \
			-movflags +faststart \
			$(RENDER_OUT)
	@echo ""
	@echo "═══ RENDERED: $(RENDER_OUT) ═══"
	@ls -lh $(RENDER_OUT)

# ── Render to GIF ────────────────────────────────────────────────────────
render-gif: ## Render headless to GIF (lower fps, palette-quantized)
	@echo "═══ RENDERING GIF @ $(RENDER_W)x$(RENDER_H) ═══"
	@echo "    Output: $(RENDER_GIF)"
	@echo ""
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
	@echo ""
	@echo "═══ RENDERED: $(RENDER_GIF) ═══"
	@ls -lh $(RENDER_GIF)

# ── Clean ─────────────────────────────────────────────────────────────────
clean: ## Remove build artifacts and rendered output
	@echo "═══ CLEANING ═══"
	cargo clean
	@rm -f $(RENDER_OUT) $(RENDER_GIF)
	@echo "═══ CLEAN ═══"

# ── Check ─────────────────────────────────────────────────────────────────
check: ## Run cargo check (fast compile check)
	cargo check $(CARGO_FLAGS)

# ── Clippy ────────────────────────────────────────────────────────────────
clippy: ## Run clippy lints
	cargo clippy $(CARGO_FLAGS) -- -W clippy::all

# ── Lines of code ─────────────────────────────────────────────────────────
loc: ## Count lines of Rust source
	@echo ""
	@echo "  ┌──────────────────────────────────────┐"
	@echo "  │  LINES OF RUST (src/)                 │"
	@echo "  ├──────────────────────────────────────┤"
	@find src -name '*.rs' | sort | while read f; do \
		lines=$$(wc -l < "$$f"); \
		printf "  │  %6d  %-28s│\n" "$$lines" "$$(basename $$f)"; \
	done
	@echo "  ├──────────────────────────────────────┤"
	@total=$$(find src -name '*.rs' -exec cat {} + | wc -l); \
		printf "  │  %6d  %-28s│\n" "$$total" "TOTAL"
	@echo "  └──────────────────────────────────────┘"
	@echo ""
