# frozen_string_literal: true

module Liquid
  class PartialCache
    def self.load(template_name, context:, parse_context:, parse_options: {})
      cached_partials = context.registers[:cached_partials]
      parse_options = parse_options.dup
      cache_key = "#{template_name}:#{parse_context.error_mode}"
      cache_key = "#{cache_key}:#{parse_options.hash}" unless parse_options.empty?
      cached = cached_partials[cache_key]
      return cached if cached

      source_cache = context.registers.static[:literal_partial_source_cache] ||= {}
      file_system = context.registers[:file_system]
      source = source_cache.fetch(template_name) do
        source_cache[template_name] = file_system.read_template_file(template_name)
      end

      parse_context.partial = true

      template_factory = context.registers[:template_factory]
      template = template_factory.for(template_name)

      begin
        partial = template.parse(
          source,
          line_numbers: !parse_context.line_number.nil?,
          error_mode: parse_context.error_mode,
          environment: parse_context.environment,
          include_options_blacklist: parse_context.partial_options[:include_options_blacklist],
          **parse_options
        )
      rescue Liquid::Error => error
        error.template_name = template&.name || template_name
        raise error
      end

      partial.name ||= template_name
      cached_partials[cache_key] = partial
    ensure
      parse_context.partial = false
    end
  end
end
