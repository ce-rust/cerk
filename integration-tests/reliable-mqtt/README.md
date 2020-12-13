# Reliable MQTT Integration Test

1. start infrastructure: `docker-compose up -d unlimited limited cerk`
  - run `docker-compose logs -f cerk` to wait until the compilation of the router endend and it started successfully.
2. run tests: `docker-compose run test-executor`
  - between test runs, `docker-compose restart cerk limited unlimited` to be executed to reset the setup
