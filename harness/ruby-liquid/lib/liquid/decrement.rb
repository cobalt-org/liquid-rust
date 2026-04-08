# frozen_string_literal: true

module Liquid
  class Decrement < Tag
    attr_reader :variable_name

    def initialize(tag_name, markup, options)
      super
      @variable_name = markup.strip
    end

    def render_to_output_buffer(context, output)
      counter_environment = context.environments.first
      value = counter_environment[@variable_name] || 0
      value -= 1
      counter_environment[@variable_name] = value
      output << value.to_s
      output
    end
  end
end
