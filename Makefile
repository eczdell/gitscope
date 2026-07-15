# ═══════════════════════════════════════════════════════════════════════════
# gitscope — Interactive Git Tree Visualizer (TUI)
# ═══════════════════════════════════════════════════════════════════════════

BINARY    := gitscope
CARGO     := cargo
RELEASE   := target/release/$(BINARY)

# Installation prefix — override with `PREFIX=~/.local make install`
PREFIX    ?= /usr/local
BINDIR    ?= $(PREFIX)/bin

.PHONY: all build install install-local install-bin uninstall clean run help

all: build

# ─── Build ────────────────────────────────────────────────────────────────

build:
	$(CARGO) build --release

build-debug:
	$(CARGO) build

# ─── Run (debug build) ────────────────────────────────────────────────────

run: build-debug
	cargo run

# ─── Install (build + copy) ──────────────────────────────────────────────

install: build
	install -d "$(DESTDIR)$(BINDIR)"
	install -m 755 $(RELEASE) "$(DESTDIR)$(BINDIR)/$(BINARY)"
	@echo ""
	@echo "  ✓ $(BINARY) installed to $(DESTDIR)$(BINDIR)/$(BINARY)"
	@echo "  ✓ Make sure $(DESTDIR)$(BINDIR) is in your PATH."
	@echo "  ✓ Run with: $(BINARY)"
	@echo ""

# ─── Install to ~/.local/bin (convenience alias) ──────────────────────────

install-local: PREFIX := $(HOME)/.local
install-local: install
	@echo "  ✓ (user-local install to $(HOME)/.local/bin)"

# ─── Install binary only (no build) — useful with sudo ───────────────────

install-bin: $(RELEASE)
	install -d "$(DESTDIR)$(BINDIR)"
	install -m 755 $(RELEASE) "$(DESTDIR)$(BINDIR)/$(BINARY)"
	@echo ""
	@echo "  ✓ $(BINARY) installed to $(DESTDIR)$(BINDIR)/$(BINARY)"
	@echo ""

# ─── Uninstall ────────────────────────────────────────────────────────────

uninstall:
	rm -f "$(DESTDIR)$(BINDIR)/$(BINARY)"
	@echo ""
	@echo "  ✓ $(BINARY) removed from $(DESTDIR)$(BINDIR)/$(BINARY)"
	@echo ""

# ─── Watch (auto-rebuild + rerun on changes, like nodemon) ──────────

watch:
	$(CARGO) watch -x run

# ─── Clean ────────────────────────────────────────────────────────────────

clean:
	$(CARGO) clean

# ─── Help ─────────────────────────────────────────────────────────────────

help:
	@echo "gitscope — Interactive Git Tree Visualizer"
	@echo ""
	@echo "Usage:"
	@echo "  make                Build the release binary"
	@echo "  make run            Build (debug) and run the TUI"
	@echo "  make install        Build and install globally"
	@echo "                      (default: PREFIX=/usr/local)"
	@echo "  make install-local  Build and install to ~/.local/bin"
	@echo "  make install-bin    Copy pre-built binary (no build step)"
	@echo ""
	@echo "Options:"
	@echo "  PREFIX=~/.local    Install to user-local prefix"
	@echo "  DESTDIR=/staging   Staging directory (for packaging)"
	@echo ""
	@echo "Examples:"
	@echo "  make install                  # install to /usr/local/bin"
	@echo "  make install-local            # install to ~/.local/bin"
	@echo "  make build && sudo make install-bin  # system-wide install"
	@echo "  sudo make install-bin         # same (if binary already built)"
	@echo "  make uninstall                # remove from /usr/local/bin"
	@echo "  make uninstall PREFIX=~/.local      # remove from ~/.local/bin"
	@echo ""

