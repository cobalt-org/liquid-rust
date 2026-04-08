.PHONY: all test clean harness-compile harness-test

CARGO ?= cargo
HARNESS_RUNNER := tests/harness/run_shopify_liquid_harness_tests.sh
RUBY_TEST_ARGS :=

ifdef ARGS
	RUBY_TEST_ARGS += $(ARGS)
endif

all: harness-compile

test: harness-test

clean:
	rm -f harness/ruby-liquid/lib/liquid/liquid_ext.bundle
	rm -f harness/ruby-liquid/lib/liquid/liquid_ext.so
	rm -f harness/ruby-liquid/lib/liquid/.liquid_ext_ruby_version

harness-test:
	$(CARGO) check --workspace
ifdef TEST
	FORCE_REBUILD_HARNESS=1 bash $(HARNESS_RUNNER) --test "$(TEST)" $(if $(strip $(RUBY_TEST_ARGS)),-- $(RUBY_TEST_ARGS),)
else
	FORCE_REBUILD_HARNESS=1 bash $(HARNESS_RUNNER) $(if $(strip $(RUBY_TEST_ARGS)),-- $(RUBY_TEST_ARGS),)
endif

harness-compile:
	FORCE_REBUILD_HARNESS=1 bash $(HARNESS_RUNNER) --compile-only
