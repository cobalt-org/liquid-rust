#!/usr/bin/env bash
# Usage:
#   bash tests/harness/run_shopify_liquid_harness_tests.sh
#   bash tests/harness/run_shopify_liquid_harness_tests.sh --compile-only
#   bash tests/harness/run_shopify_liquid_harness_tests.sh --test test/integration/template_test.rb
#
# Pinned baseline:
#   Ruby 3.4.1
#   Shopify Liquid commit a9c85622ddd784078c2eed34b19a351fe57362cf

set -euo pipefail

repo_root="$(
  cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." >/dev/null 2>&1
  pwd
)"

repo_parent="$(cd -- "${repo_root}/.." >/dev/null 2>&1 && pwd)"
sibling_shopify_root="${repo_parent}/liquid"
alternate_shopify_root="${repo_parent}/shopify-liquid"
shopify_repo_url="${SHOPIFY_LIQUID_REPO_URL:-https://github.com/shopify/liquid.git}"
ruby_version="${RBENV_VERSION:-3.4.1}"

resolve_rbenv_bin() {
  if [[ -n "${RBENV_BIN:-}" ]]; then
    printf '%s\n' "${RBENV_BIN}"
    return 0
  fi

  if command -v rbenv >/dev/null 2>&1; then
    command -v rbenv
    return 0
  fi

  if [[ -x "/opt/homebrew/bin/rbenv" ]]; then
    printf '%s\n' "/opt/homebrew/bin/rbenv"
    return 0
  fi

  if [[ -x "${HOME}/.rbenv/bin/rbenv" ]]; then
    printf '%s\n' "${HOME}/.rbenv/bin/rbenv"
    return 0
  fi

  return 1
}

rbenv_bin=""
if rbenv_candidate="$(resolve_rbenv_bin 2>/dev/null)"; then
  rbenv_bin="${rbenv_candidate}"
fi

if [[ -n "${SHOPIFY_LIQUID_ROOT:-}" ]]; then
  shopify_root="${SHOPIFY_LIQUID_ROOT}"
elif [[ -d "${sibling_shopify_root}/test" ]]; then
  shopify_root="${sibling_shopify_root}"
elif [[ -d "${alternate_shopify_root}/test" ]]; then
  shopify_root="${alternate_shopify_root}"
else
  shopify_root="${sibling_shopify_root}"
fi

ruby_bin="${RUBY_BIN:-}"
ruby_bundle="${BUNDLE_BIN:-}"
ruby_bin_desc=""
ruby_bundle_desc=""
bundle_cmd_prefix=()

if [[ -n "${rbenv_bin}" ]]; then
  if [[ -z "${ruby_bin}" ]]; then
    if ruby_candidate="$(RBENV_VERSION="${ruby_version}" "${rbenv_bin}" which ruby 2>/dev/null)"; then
      ruby_bin="${ruby_candidate}"
    fi
  fi

  if [[ -z "${ruby_bundle}" ]] && RBENV_VERSION="${ruby_version}" "${rbenv_bin}" which bundle >/dev/null 2>&1; then
    bundle_cmd_prefix=("${rbenv_bin}" exec bundle)
    ruby_bundle_desc="rbenv exec bundle (${ruby_version})"
  fi
fi

ruby_bin="${ruby_bin:-ruby}"
ruby_bundle="${ruby_bundle:-bundle}"
ruby_bin_desc="${ruby_bin_desc:-${ruby_bin}}"

if ((${#bundle_cmd_prefix[@]} == 0)); then
  bundle_cmd_prefix=("${ruby_bundle}")
  ruby_bundle_desc="${ruby_bundle_desc:-${ruby_bundle}}"
fi

bootstrap="${repo_root}/harness/bootstrap.rb"
harness_gem_root="${repo_root}/harness/ruby-liquid"
rb_sys_target_dir="${RB_SYS_CARGO_TARGET_DIR:-/tmp/liquid-ruby-ext-target}"
extension_dir="${harness_gem_root}/lib/liquid"
extension_stamp="${extension_dir}/.liquid_ext_ruby_version"
max_attempts="${HARNESS_TEST_RETRIES:-3}"
bundle_install_attempted=0
bundle_checkout_error_pattern='is not yet checked out\. Run `bundle install` first\.'


selected_test_files=()
ruby_test_args=()
compile_only=0

if [[ ! -f "${bootstrap}" ]]; then
  echo "Missing bootstrap file: ${bootstrap}" >&2
  exit 1
fi

if [[ ! -d "${harness_gem_root}" ]]; then
  echo "Missing harness gem directory: ${harness_gem_root}" >&2
  exit 1
fi

if ! RBENV_VERSION="${ruby_version}" "${bundle_cmd_prefix[@]}" --version >/dev/null 2>&1; then
  echo "Missing bundle executable: ${ruby_bundle_desc}" >&2
  if [[ -n "${rbenv_bin}" ]]; then
    echo "The runner looked for Bundler via rbenv ${ruby_version} first." >&2
  fi
  echo "Set BUNDLE_BIN=/path/to/bundle to override." >&2
  exit 1
fi

if ! "${ruby_bin}" --version >/dev/null 2>&1; then
  echo "Missing ruby executable: ${ruby_bin_desc}" >&2
  if [[ -n "${rbenv_bin}" ]]; then
    echo "The runner looked for Ruby via rbenv ${ruby_version} first." >&2
  fi
  echo "Set RUBY_BIN=/path/to/ruby to override." >&2
  exit 1
fi

ensure_shopify_checkout() {
  local clone_target="${shopify_root}"

  if [[ -d "${clone_target}/test" ]]; then
    return 0
  fi

  if [[ -e "${clone_target}" && ! -d "${clone_target}/.git" ]]; then
    echo "Cannot clone Shopify Liquid into existing non-git path: ${clone_target}" >&2
    exit 1
  fi

  echo "Shopify Liquid checkout not found at ${clone_target}, cloning ${shopify_repo_url}..." >&2
  git clone "${shopify_repo_url}" "${clone_target}"
}

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --compile-only)
      compile_only=1
      shift
      ;;
    --test)
      if [[ "$#" -lt 2 ]]; then
        echo "Missing value for --test" >&2
        exit 1
      fi
      selected_test_files+=("$2")
      shift 2
      ;;
    --)
      shift
      ruby_test_args+=("$@")
      break
      ;;
    *)
      echo "Unknown argument: $1" >&2
      echo "Usage: $0 [--compile-only] [--test path]... [-- extra ruby test args]" >&2
      exit 1
      ;;
  esac
done

if [[ "${compile_only}" != "1" ]]; then
  ensure_shopify_checkout
fi

if [[ "${compile_only}" != "1" && "${#selected_test_files[@]}" -eq 0 ]]; then
  while IFS= read -r test_file; do
    selected_test_files+=("${test_file}")
  done < <(
    find "${shopify_root}/test" -type f -name '*_test.rb' \
      -not -path '*/test_helper.rb' \
      | LC_ALL=C sort \
      | sed "s#^${shopify_root}/##"
  )
fi

if [[ "${compile_only}" != "1" && "${#selected_test_files[@]}" -eq 0 ]]; then
  echo "No upstream Shopify Liquid test files were found under ${shopify_root}/test" >&2
  exit 1
fi

needs_compile="${FORCE_REBUILD_HARNESS:-0}"

if [[ "${needs_compile}" != "1" ]] \
  && [[ ! -f "${extension_dir}/liquid_ext.bundle" ]] \
  && [[ ! -f "${extension_dir}/liquid_ext.so" ]]; then
  needs_compile="1"
fi

if [[ "${needs_compile}" != "1" ]]; then
  if [[ ! -f "${extension_stamp}" ]] || [[ "$(<"${extension_stamp}")" != "${ruby_version}" ]]; then
    needs_compile="1"
  fi
fi

if [[ "${needs_compile}" == "1" ]]; then
  echo "Ruby harness extension missing, building it..."
  (
    cd "${harness_gem_root}"
    unset RUSTC_WRAPPER
    RB_SYS_CARGO_TARGET_DIR="${rb_sys_target_dir}" \
    RBENV_VERSION="${ruby_version}" \
    "${bundle_cmd_prefix[@]}" exec rake compile
  )
  printf '%s\n' "${ruby_version}" > "${extension_stamp}"
fi

if [[ "${compile_only}" == "1" ]]; then
  exit 0
fi

ensure_shopify_bundle() {
  local reason="${1:-bundle check failed}"

  if (( bundle_install_attempted )); then
    return 1
  fi

  bundle_install_attempted=1
  echo "Shopify Liquid bundle needs installation (${reason}), running bundle install..." >&2
  RBENV_VERSION="${ruby_version}" "${bundle_cmd_prefix[@]}" install
}

cd "${shopify_root}"

if ! RBENV_VERSION="${ruby_version}" "${bundle_cmd_prefix[@]}" check >/dev/null 2>&1; then
  ensure_shopify_bundle "bundle check failed"
fi

for test_file in "${selected_test_files[@]}"; do
  echo
  echo "==> ${test_file}"
  ruby_cmd=(
    "${bundle_cmd_prefix[@]}"
    exec
    "${ruby_bin}"
    -Itest
    -r "${bootstrap}"
    "${test_file}"
  )
  if ((${#ruby_test_args[@]} > 0)); then
    ruby_cmd+=("${ruby_test_args[@]}")
  fi
  attempt=1
  while true; do
    output_file="$(mktemp "${TMPDIR:-/tmp}/shopify-harness.XXXXXX")"

    if RBENV_VERSION="${ruby_version}" "${ruby_cmd[@]}" 2>&1 | tee "${output_file}"; then
      rm -f "${output_file}"
      break
    fi

    if grep -Eq "${bundle_checkout_error_pattern}" "${output_file}"; then
      rm -f "${output_file}"
      if ensure_shopify_bundle "missing liquid-spec checkout"; then
        echo "Retrying ${test_file} after bundle install..." >&2
        continue
      fi
    else
      rm -f "${output_file}"
    fi

    if (( attempt >= max_attempts )); then
      exit 1
    fi

    attempt=$((attempt + 1))
    echo "Retrying ${test_file} (attempt ${attempt}/${max_attempts})..." >&2
  done
done
