.PHONY: help build run render install-linux set-linux status-linux unset-linux clean check clippy loc

CTL         := ./scripts/solverforge-screensaverctl
INSTALL     := ./scripts/install-linux.sh
DEMO        := solverforge-screensaver
CARGO_FLAGS := --release
RENDER_W    := 1280
RENDER_H    := 720
RENDER_FPS  := 60
RENDER_SECS := 30
RENDER_OUT  := screensaver_03.mp4
FFMPEG_VCODEC := libx264
FFMPEG_CRF    := 18
FFMPEG_PRESET := slow

help:
	@echo ""
	@echo " SOLVERFORGE SCREENSAVER // PHOSPHOR RITUAL"
	@echo ""
	@echo "  make build   Build release binary"
	@echo "  make run     Run the screensaver"
	@echo "  make render       Render headless MP4 via ffmpeg"
	@echo "  make install-linux  Build + install into ~/.local/bin"
	@echo "  make set-linux      Configure idle launch on Linux"
	@echo "  make status-linux   Show install/config status"
	@echo "  make unset-linux    Remove idle-launch configuration"
	@echo "  make check        cargo check"
	@echo "  make clippy  cargo clippy"
	@echo "  make loc     count Rust LOC"
	@echo ""

build:
	cargo build $(CARGO_FLAGS)

run:
	cargo run $(CARGO_FLAGS)

render:
	@command -v ffmpeg >/dev/null 2>&1 || { echo "ERROR: ffmpeg not found"; exit 1; }
	cargo run $(CARGO_FLAGS) -- --render $(RENDER_SECS) | \
		ffmpeg -y -f rawvideo -pixel_format bgr24 -video_size $(RENDER_W)x$(RENDER_H) -framerate $(RENDER_FPS) -i - \
		-c:v $(FFMPEG_VCODEC) -crf $(FFMPEG_CRF) -preset $(FFMPEG_PRESET) -pix_fmt yuv420p $(RENDER_OUT)

install-linux:
	$(INSTALL)

set-linux:
	$(CTL) set --timeout 300

status-linux:
	$(CTL) status

unset-linux:
	$(CTL) unset

clean:
	cargo clean
	@rm -f $(RENDER_OUT)

check:
	cargo check $(CARGO_FLAGS)

clippy:
	cargo clippy $(CARGO_FLAGS) -- -W clippy::all

loc:
	@find src -name '*.rs' -exec cat {} + | wc -l
