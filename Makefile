ARGS:=-p ci -f dockerfiles/docker-compose.base.yml -f dockerfiles/docker-compose.ci.yml

.PHONY: start-test-env
start-test-env:
	docker-compose $(ARGS) up --remove-orphans --detach

.PHONY: stop-test-env
stop-test-env:
	docker-compose $(ARGS) down --remove-orphans --volumes
