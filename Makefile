CARGO ?= cargo
BINARY := middleauth-ssh-proxy

SOURCES := $(shell find src/ -type f -name '*.rs')
CARGO_SOURCES := Cargo.toml .cargo/config.toml

# Target Configurations
MACOS_BUILD_TARGETS := x86_64-apple-darwin aarch64-apple-darwin
MACOS_TARGETS       := $(MACOS_BUILD_TARGETS) universal-apple-darwin
LINUX_TARGETS       := x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
WINDOWS_TARGETS     := x86_64-pc-windows-gnu

UNIX_BUILD_TARGETS  := $(MACOS_BUILD_TARGETS) $(LINUX_TARGETS)
TARGETS             := $(MACOS_TARGETS) $(LINUX_TARGETS) $(WINDOWS_TARGETS)

# Binary Paths mapping
LINUX_TARGET_BINARIES   := $(patsubst %,target/%/release/$(BINARY),$(LINUX_TARGETS))
MACOS_TARGET_BINARIES   := $(patsubst %,target/%/release/$(BINARY),$(MACOS_BUILD_TARGETS))
WINDOWS_TARGET_BINARIES := $(patsubst %,target/%/release/$(BINARY).exe,$(WINDOWS_TARGETS))
TARGET_BINARIES         := $(MACOS_TARGET_BINARIES) $(LINUX_TARGET_BINARIES) $(WINDOWS_TARGET_BINARIES)

.PHONY: clean default native-debug native-release release release-macos release-linux release-windows $(TARGETS)

default: native-debug

# Shortcut aliases to match your targets list
$(UNIX_BUILD_TARGETS): %: target/%/release/$(BINARY)
$(WINDOWS_TARGET_TARGETS): %: target/%/release/$(BINARY).exe

# ==============================================================================
# STATIC PATTERN RULES (Replaces foreach/eval/call)
# ==============================================================================

# Matches all UNIX binary file paths and extracts the target architecture via %
$(LINUX_TARGET_BINARIES) $(MACOS_TARGET_BINARIES): target/%/release/$(BINARY): $(SOURCES) $(CARGO_SOURCES)
	@echo "Running build for target: $*"
	$(CARGO) build --target=$* --release

# Matches all Windows binary file paths
$(WINDOWS_TARGET_BINARIES): target/%/release/$(BINARY).exe: $(SOURCES) $(CARGO_SOURCES)
	@echo "Running build for target: $*"
	$(CARGO) build --target=$* --release

# ==============================================================================
# STANDARD & COMBINED TARGETS
# ==============================================================================

native-debug: target/debug/$(BINARY)
target/debug/$(BINARY): $(SOURCES) $(CARGO_SOURCES)
	$(CARGO) build

native-release: target/release/$(BINARY)
target/release/$(BINARY): $(SOURCES) $(CARGO_SOURCES)
	$(CARGO) build --release

universal-apple-darwin: target/universal-apple-darwin/release/$(BINARY)
target/universal-apple-darwin/release/$(BINARY): $(MACOS_TARGET_BINARIES)
	@mkdir -p target/universal-apple-darwin/release
	lipo -create -o $@ $^

# Meta compilation targets
release: $(TARGETS)
release-macos: $(MACOS_TARGET_BINARIES) universal-apple-darwin
release-linux: $(LINUX_TARGET_BINARIES)
release-windows: $(WINDOWS_TARGET_BINARIES)

clean:
	rm -rf target
