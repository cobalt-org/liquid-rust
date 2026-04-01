# frozen_string_literal: true

module Liquid
  class VariableLookup
    attr_reader :name, :lookups, :command_flags

    def initialize(name, lookups = [], command_flags: [])
      @name = name
      @lookups = Array(lookups)
      @command_flags = Array(command_flags)
    end
  end
end
