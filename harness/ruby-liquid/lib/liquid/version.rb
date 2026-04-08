# frozen_string_literal: true

module Liquid
  # Match the pinned upstream compatibility target and avoid duplicate-constant
  # warnings if some code path loaded liquid/version before the harness entrypoint.
  VERSION = "5.12.0" unless const_defined?(:VERSION, false)
end
