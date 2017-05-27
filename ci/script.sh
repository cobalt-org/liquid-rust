# This script takes care of testing your crate

set -ex

main() {
    cross check --verbose --target $TARGET
    cross check --verbose --target $TARGET --features "cli"
    cross check --verbose --target $TARGET --features "cli serde_yaml serde_json"
    cross fmt -- --write-mode=diff

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --verbose --target $TARGET --features "cli serde_yaml serde_json"
    if [ "$TRAVIS_RUST_VERSION" = "nightly" ]; then (cargo clippy --features "cli serde_yaml serde_json") fi
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
