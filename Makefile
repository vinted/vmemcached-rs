ARGS:=-p ci -f dockerfiles/docker-compose.base.yml -f dockerfiles/docker-compose.ci.yml

.PHONY: start-test-env
start-test-env:
	docker-compose $(ARGS) up --remove-orphans --detach --force-recreate

.PHONY: stop-test-env
stop-test-env:
	docker-compose $(ARGS) down --remove-orphans --volumes

.PHONY: test
test: start-test-env
	cargo test

.PHONY: start-dev-env
start-dev-env:
	docker-compose $(ARGS) up --remove-orphans
