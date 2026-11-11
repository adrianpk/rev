.PHONY: lint test-hooks install-hooks

lint:
	bash scripts/check-quality.sh

test-hooks:
	bash scripts/test-hooks.sh

install-hooks:
	bash scripts/install-hooks.sh
