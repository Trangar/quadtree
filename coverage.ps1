podman build ../../scripts/ --file ../../scripts/coverage.Dockerfile --tag rust_coverage
podman run -v ${pwd}:/app -it rust_coverage

