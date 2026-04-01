# frozen_string_literal: true

module Liquid
  class ParseContext
    attr_accessor :line_number, :locale, :environment, :error_mode, :warnings, :depth

    def initialize(environment: Liquid::Environment.default, line_number: 1, locale: nil, error_mode: nil, warnings: [])
      @environment = environment
      @line_number = line_number
      @locale = locale || Liquid::I18n.new
      @error_mode = (error_mode || environment.error_mode).to_sym
      @warnings = warnings
      @depth = 0
    end
  end
end
