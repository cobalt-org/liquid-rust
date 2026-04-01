# frozen_string_literal: true

module Liquid
  class Template
    class ExceptionRendererRaised < StandardError
      attr_reader :error

      def initialize(error)
        @error = error
        super(error.message)
        set_backtrace(error.backtrace)
      end
    end

    RENDER_OPTION_KEYS = [
      :filters,
      :registers,
      :exception_renderer,
      :global_filter,
      :strict_variables,
      :strict_filters,
      :output
    ].freeze

    attr_accessor :name, :assigns, :instance_assigns
    attr_reader :registers
    attr_reader :resource_limits, :warnings, :errors, :root

    class << self
      def parse(source, line_numbers: false, error_mode: nil, environment: nil, **_options)
        new(environment: environment || Liquid::Environment.default).parse(
          source,
          line_numbers: line_numbers,
          error_mode: error_mode,
          environment: environment
        )
      end

      def error_mode=(mode)
        Liquid::Environment.default.error_mode = mode
      end

      def error_mode
        Liquid::Environment.default.error_mode
      end

      def file_system=(file_system)
        Liquid::Environment.default.file_system = file_system
      end

      def file_system
        Liquid::Environment.default.file_system
      end

      def tags
        Liquid::Environment.default.tags
      end

      def register_tag(name, klass)
        Liquid::Environment.default.register_tag(name, klass)
      end

      def register_filter(mod)
        Liquid::Environment.default.register_filter(mod)
      end
    end

    def initialize(environment: Liquid::Environment.default)
      @environment = environment
      @assigns = {}
      @instance_assigns = {}
      @errors = []
      @warnings = []
      @registers = Liquid::Registers.new
      @resource_limits = Liquid::ResourceLimits.new
      @root = Liquid::BlockBody.new([])
      @handle = nil
    end

    def parse(source, line_numbers: false, error_mode: nil, environment: nil, **_options)
      env = environment || @environment
      effective_error_mode = (error_mode || env&.error_mode || :strict).to_sym
      normalized_source = normalize_source(source.to_s, effective_error_mode)
      @handle = Liquid::RustExtension.ext_parse(normalized_source, line_numbers, error_mode&.to_s, env&.native_handle)
      @errors = Array(Liquid::RustExtension.ext_template_errors(@handle))
      @warnings = Array(Liquid::RustExtension.ext_template_warnings(@handle))
      @root = Liquid::BlockBody.from_native(Liquid::RustExtension.ext_template_root(@handle))
      self
    rescue StandardError => error
      raise Liquid::SyntaxError.wrap(error, default_class: Liquid::SyntaxError)
    end

    def render(*args)
      render_args = args.dup
      options = extract_render_options(render_args)
      context = build_render_context(render_args)
      apply_options_to_context(context, options)

      rendered = Liquid::RustExtension.ext_render(@handle, context.native_handle)
      @errors = collect_template_errors
      rendered = finalize_render_output(rendered, context)
      options[:output] ? options[:output] << rendered : rendered
    rescue ExceptionRendererRaised => error
      @errors = collect_template_errors
      raise error.error
    rescue StandardError => error
      wrapped = Liquid::Error.wrap(error)
      @errors = [wrapped]
      rendered = wrapped.to_s
      options[:output] ? options[:output] << rendered : rendered
    end

    def render!(*args)
      render_args = args.dup
      options = extract_render_options(render_args)
      context = build_render_context(render_args)
      apply_options_to_context(context, options)

      rendered = Liquid::RustExtension.ext_render_strict(@handle, context.native_handle)
      rendered = context.apply_global_filter(rendered)
      @errors = collect_template_errors
      options[:output] ? options[:output] << rendered : rendered
    rescue StandardError => error
      raise error unless liquid_error?(error)

      raise Liquid::Error.wrap(error)
    end

    def render_to_output_buffer(context, output)
      render(context, output: output)
    end

    private

    def build_render_context(args)
      case args.first
      when Liquid::Context
        args.shift
      when Liquid::Drop
        drop = args.shift
        drop.context = Liquid::Context.new([@assigns, drop], registers: @registers, environment: @environment)
      when Hash
        Liquid::Context.new([@assigns, args.shift], registers: @registers, environment: @environment)
      when nil
        Liquid::Context.new(@assigns.dup, registers: @registers, environment: @environment)
      else
        raise ::ArgumentError, "Expected Hash, Liquid::Drop, Liquid::Context, or nil as parameter"
      end
    end

    def extract_render_options(args)
      case args.last
      when Hash
        return {} unless render_options_hash?(args)

        args.pop.dup
      when Module, Array
        { filters: args.pop }
      else
        {}
      end
    end

    def apply_options_to_context(context, options)
      context.add_filters(options[:filters]) if options.key?(:filters) && !options[:filters].nil?
      Array(options[:registers]).each do |key, value|
        context.registers[key] = value
      end
      context.exception_renderer = options[:exception_renderer] if options.key?(:exception_renderer)
      context.native_handle["exception_renderer"] = context.exception_renderer || context.environment&.exception_renderer
      context.global_filter = options[:global_filter] if options.key?(:global_filter)
      context.strict_variables = options[:strict_variables] if options.key?(:strict_variables)
      context.strict_filters = options[:strict_filters] if options.key?(:strict_filters)
      context.template_name ||= name
    end

    def normalize_source(source, error_mode)
      source.gsub(/\{\{(.*?)\}\}/m) do
        "{{#{normalize_output_markup(Regexp.last_match(1), error_mode)}}}"
      end
    end

    def normalize_output_markup(markup, error_mode)
      normalized = markup.gsub(/(?<=\w|\]|\))\s*\.\s*(?=\w)/, ".")

      case error_mode
      when :strict2, :lax
        normalized = normalized.gsub(/,\s*(?=\||\z)/, "")
        normalized = normalized.gsub(/:\s*(?=\||\z)/, "")
      end

      return normalized unless error_mode == :lax

      normalized = normalized.sub(/\A(\s*)\[\s*\[/, '\1[')
      opens = normalized.count("[")
      closes = normalized.count("]")
      normalized += "]" * (opens - closes) if opens > closes
      normalized
    end

    def liquid_error?(error)
      return true if error.is_a?(Liquid::Error)

      error.message.to_s.start_with?("liquid:")
    end

    def collect_template_errors
      Array(Liquid::RustExtension.ext_template_errors(@handle)).map do |error|
        error.is_a?(Liquid::Error) ? error : Liquid::Error.wrap(::RuntimeError.new(error.to_s))
      end
    end

    def finalize_render_output(rendered, context)
      context.apply_global_filter(rendered)
    end

    def render_options_hash?(args)
      return false unless args.last.is_a?(Hash)
      return true if args.length > 1
      return false if args.last.empty?

      args.last.keys.all? do |key|
        key.respond_to?(:to_sym) && RENDER_OPTION_KEYS.include?(key.to_sym)
      end
    end
  end
end
