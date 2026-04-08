# frozen_string_literal: true

module Liquid
  class ParseContext
    attr_accessor :line_number, :locale, :environment, :depth, :trim_whitespace
    attr_reader :error_mode, :warnings, :partial

    def initialize(options = nil, environment: Liquid::Environment.default, line_number: 1, locale: nil, error_mode: nil, warnings: [], **extra_options)
      options =
        if options.is_a?(Hash)
          options.merge(extra_options)
        elsif extra_options.empty?
          options
        else
          extra_options
        end

      if options.is_a?(Hash)
        @template_options = options.dup
        environment = options.fetch(:environment, environment)
        line_number = options.fetch(:line_number, line_number)
        locale = options.fetch(:locale, locale)
        error_mode = options.fetch(:error_mode, error_mode)
        warnings = options.fetch(:warnings, warnings)
      else
        @template_options = {}
      end

      @environment = environment
      @line_number = line_number
      @locale = locale || Liquid::I18n.new
      @warnings = warnings
      @depth = 0
      @string_scanner = StringScanner.new("")
      @expression_cache =
        if @template_options[:expression_cache].nil?
          {}
        elsif @template_options[:expression_cache].respond_to?(:[]) &&
            @template_options[:expression_cache].respond_to?(:[]=)
          @template_options[:expression_cache]
        elsif @template_options[:expression_cache]
          {}
        end
      @template_options[:environment] ||= @environment
      @template_options[:line_number] ||= @line_number
      @template_options[:locale] ||= @locale
      @template_options[:error_mode] = error_mode if error_mode
      self.partial = false
    end

    def [](option_key)
      @template_options[option_key]
    end

    def new_block_body
      Liquid::BlockBody.new([])
    end

    def new_tokenizer(source, start_line_number: nil, for_liquid_tag: false)
      Liquid::Tokenizer.new(
        source: source,
        string_scanner: @string_scanner,
        line_numbers: !line_number.nil?,
        line_number: start_line_number || line_number,
        for_liquid_tag: for_liquid_tag
      )
    end

    def new_parser(input)
      @string_scanner.string = input
      Parser.new(@string_scanner)
    end

    def safe_parse_expression(parser)
      Expression.safe_parse(parser, @string_scanner, @expression_cache)
    end

    def parse_expression(markup, safe: false)
      if !safe && @error_mode == :strict2
        raise Liquid::InternalError, "unsafe parse_expression cannot be used in strict2 mode"
      end

      Expression.parse(markup, @string_scanner, @expression_cache)
    end

    def partial=(value)
      @partial = value
      options = value ? partial_options : @template_options
      fallback_error_mode = value ? Liquid::Environment.default.error_mode : @environment.error_mode
      @error_mode = (options[:error_mode] || fallback_error_mode).to_sym
    end

    def partial_options
      @partial_options ||= begin
        dont_pass = @template_options[:include_options_blacklist]
        if dont_pass == true
          { locale: @locale }
        elsif dont_pass.is_a?(Array)
          @template_options.reject { |key, _value| dont_pass.include?(key) }
        else
          @template_options
        end
      end
    end
  end
end
