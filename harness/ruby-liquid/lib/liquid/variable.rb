# frozen_string_literal: true

module Liquid
  class Variable
    attr_reader :name, :filters, :line_number

    def initialize(name, filters: [], line_number: 1)
      @name = name
      @filters = Array(filters)
      @line_number = line_number
    end
  end
end
