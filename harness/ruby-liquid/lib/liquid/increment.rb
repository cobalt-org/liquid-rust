# frozen_string_literal: true

module Liquid
  class Increment < Tag
    attr_reader :variable_name

    def initialize(tag_name, markup, options)
      super
      @variable_name = markup.strip
    end

    def render_to_output_buffer(context, output)
      counter_environment = context.environments.first
      value = counter_environment[@variable_name] || 0
      counter_environment[@variable_name] = value + 1
      output << value.to_s
      output
    end
  end
end
