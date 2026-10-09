CARGO ?= cargo
BINARY=middleauth-ssh-proxy

SOURCES := $(shell find src/ -type f -name '*.rs')
CARGO_SOURCES := Cargo.toml .cargo/config.toml

MACOS_BUILD_TARGETS = x86_64-apple-darwin aarch64-apple-darwin
MACOS_TARGETS = $(MACOS_BUILD_TARGETS) universal-apple-darwin
LINUX_TARGETS = x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
WINDOWS_TARGETS = x86_64-pc-windows-gnu #aarch64-pc-windows-gnullvm

UNIX_BUILD_TARGETS = ${MACOS_BUILD_TARGETS} ${LINUX_TARGETS}
BUILD_TARGETS = ${UNIX_BUILD_TARGETS} ${WINDOWS_TARGETS}
TARGETS = $(MACOS_TARGETS) ${LINUX_TARGETS} ${WINDOWS_TARGETS}

LINUX_TARGET_BINARIES = $(patsubst %,target/%/release/${BINARY},${LINUX_TARGETS})
MACOS_TARGET_BINARIES = $(patsubst %,target/%/release/${BINARY},${MACOS_BUILD_TARGETS})
UNIX_TARGET_BINARIES = ${LINUX_TARGET_BINARIES} ${MACOS_TARGET_BINARIES}
WINDOWS_TARGET_BINARIES = $(patsubst %,target/%/release/${BINARY}.exe,${WINDOWS_TARGETS})
TARGET_BINARIES = ${UNIX_TARGET_BINARIES} ${WINDOWS_TARGET_BINARIES}

.PHONY: clean

default: native-debug

native-debug: target/debug/${BINARY}
native-release: target/release/${BINARY}

target/debug/${BINARY}: ${SOURCES} ${CARGO_SOURCES}
	$(CARGO) build

target/release/${BINARY}: ${SOURCES} ${CARGO_SOURCES}
	$(CARGO) build --release


release: $(TARGETS)
release-macos: $(MACOS_TARGETS)
release-linux: $(LINUX_TARGETS)
release-windows: $(WINDOWS_TARGETS)

universal-apple-darwin: target/universal-apple-darwin/release/${BINARY}
target/universal-apple-darwin/release/${BINARY}: $(MACOS_TARGET_BINARIES)
	mkdir -p target/universal-apple-darwin/release
	lipo -create -o target/universal-apple-darwin/release/${BINARY} ${MACOS_TARGET_BINARIES}

$(UNIX_BUILD_TARGETS): target/%/release/${BINARY}: ${SOURCES} ${CARGO_SOURCES}
	@echo "Running job for target: $*"
    $(CARGO) build --target=$* --release

$(WINDOWS_TARGETS): target/%/release/${BINARY}.exe: ${SOURCES} ${CARGO_SOURCES}
	@echo "Running job for target: $*"
    $(CARGO) build --target=$* --release

clean:
	rm -r target