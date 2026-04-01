# frozen_string_literal: true

require "json"

module Liquid
  class TestReporter
    def initialize(io = $stdout)
      @io = io
    end

    def record(event, payload = {})
      @io.puts(JSON.generate({ event: event }.merge(payload)))
    end
  end
end
