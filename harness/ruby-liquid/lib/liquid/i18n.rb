# frozen_string_literal: true

require "yaml"

module Liquid
  class I18n
    DEFAULT_LOCALE = File.join(File.expand_path(__dir__), "locales", "en.yml")

    TranslationError = Class.new(StandardError)

    attr_reader :path

    def initialize(path = DEFAULT_LOCALE)
      @path = path
    end

    def translate(name, vars = {})
      interpolate(deep_fetch_translation(name), vars)
    rescue TranslationError
      raise
    rescue StandardError
      fallback = fallback_translation(name)
      raise TranslationError, "Translation for #{name} does not exist in locale #{path}" if fallback.nil?

      interpolate(fallback, vars)
    end
    alias_method :t, :translate

    def locale
      @locale ||= YAML.load_file(@path)
    end

    private

    def interpolate(template, vars)
      template.gsub(/%\{(\w+)\}/) do
        vars[Regexp.last_match(1).to_sym].to_s
      end
    end

    def deep_fetch_translation(name)
      name.to_s.split(".").reduce(locale) do |level, key|
        if level.respond_to?(:key?) && level.key?(key)
          level[key]
        else
          raise TranslationError, "Translation for #{name} does not exist in locale #{path}"
        end
      end
    end

    def fallback_translation(name)
      FALLBACK_TRANSLATIONS[name.to_s]
    end

    FALLBACK_TRANSLATIONS = {
      "errors.syntax.render" => "Error in tag 'render' - Valid syntax: render '[template]' (with|for) [object|collection] (as [alias])?",
    }.freeze
  end
end
