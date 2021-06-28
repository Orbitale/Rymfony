.DEFAULT_GOAL:=help

##
## Rymfony dev
## ───────────
##

# Helper vars for pretty display
_TITLE := "\033[32m[%s]\033[0m %s\n"
_ERROR := "\033[31m[%s]\033[0m %s\n"

help: ## ❓ Show this help.
	@printf "\n Available commands:\n\n"
	@grep -E '(^[a-zA-Z_-]+:.*?##.*$$)|(^##)' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-25s\033[0m %s\n", $$1, $$2}' | sed -e 's/\[32m## */[33m/'
.PHONY: help

build: ## Build the project for development.
	@cargo build
.PHONY: build

release: ## Build the project for release.
	@cargo build --release
.PHONY: release

test: build ## Run the tests
	@cargo test
	@./target/debug/rymfony serve --daemon && sleep 3 && \
	./tests/bats/bin/bats tests ; \
	./target/debug/rymfony stop
.PHONY: test