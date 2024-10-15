VIRTLINT_BASE = $(shell realpath ../target/debug 2>/dev/null)
#some dummy fallback
ifeq ($(VIRTLINT_BASE),)
VIRTLINT_BASE := $(PWD)/target/debug/
endif

VIRT_LINT_LUA_PATH = $(shell realpath "../validators_lua/" 2>/dev/null)
VIRT_LINT_PYTHON_PATH = $(shell realpath "../validators_python/" 2>/dev/null)

run:
	PYTHONPATH="${VIRTLINT_BASE}" VIRT_LINT_LUA_PATH="${VIRT_LINT_LUA_PATH}" VIRT_LINT_PYTHON_PATH="${VIRT_LINT_PYTHON_PATH}" ./virt_lint_test.py
