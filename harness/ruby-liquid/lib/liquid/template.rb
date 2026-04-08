# frozen_string_literal: true

module Liquid
  class Template
    NATIVE_CONFORMANCE_TAGS = [].freeze

    class ExceptionRendererRaised < StandardError
      attr_reader :error

      def initialize(error)
        @error = error
        super(error.message)
        set_backtrace(error.backtrace)
      end
    end

    class MergedAssigns
      def initialize(assigns, instance_assigns)
        @assigns = assigns
        @instance_assigns = instance_assigns
      end

      def key?(key)
        @instance_assigns.key?(key) || @assigns.key?(key)
      end

      def [](key)
        if @assigns.key?(key)
          @assigns[key]
        else
          @instance_assigns[key]
        end
      end

      def []=(key, value)
        if @assigns.key?(key)
          @assigns[key] = value
        elsif @instance_assigns.key?(key)
          @instance_assigns[key] = value
        else
          @instance_assigns[key] = value
        end
      end
    end

    class CustomTagRegistryDrop < Liquid::Drop
      def initialize(template, custom_tags)
        super()
        @template = template
        @custom_tags = custom_tags.each_with_object({}) do |custom_tag, acc|
          next unless custom_tag[:method_name]

          acc[custom_tag[:method_name]] = custom_tag
        end
      end

      def liquid_method_missing(method)
        custom_tag = @custom_tags[method.to_s]
        return super unless custom_tag

        @template.send(:render_custom_tag, custom_tag, @context)
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
    attr_reader :resource_limits, :warnings, :errors, :root, :profiler

    class << self
      def parse(source, options = nil, line_numbers: false, error_mode: nil, environment: nil, **kwargs)
        parse_options = normalize_parse_options(options, kwargs)
        parse_context = parse_options.delete(:parse_context)
        env = parse_context&.environment || parse_options[:environment] || environment || Liquid::Environment.default

        new(environment: env).parse(
          source,
          parse_context || parse_options,
          line_numbers: line_numbers,
          error_mode: error_mode,
          environment: env,
          **(parse_context ? parse_options : {})
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

      private

      def normalize_parse_options(options, kwargs)
        if options.is_a?(Liquid::ParseContext)
          kwargs.merge(parse_context: options)
        elsif options.is_a?(Hash)
          options.merge(kwargs)
        else
          kwargs
        end
      end
    end

    def initialize(environment: Liquid::Environment.default)
      @environment = environment
      @assigns = {}
      @instance_assigns = {}
      @errors = []
      @warnings = []
      @registers = Liquid::Registers.new
      @resource_limits = Liquid::ResourceLimits.new(@environment.default_resource_limits)
      @root = Liquid::BlockBody.new([])
      @handle = nil
      @line_numbers_enabled = false
      @source = nil
      @custom_tags = []
      @custom_tag_registry_name = nil
      @custom_tag_registry = nil
      @compatibility_render_errors = []
      @profiling = false
    end

    def parse(source, options = nil, line_numbers: false, error_mode: nil, environment: nil, **kwargs)
      options =
        if options.is_a?(Liquid::ParseContext)
          kwargs.merge(parse_context: options)
        elsif options.is_a?(Hash)
          options.merge(kwargs)
        else
          kwargs
        end

      parse_context = options.delete(:parse_context)
      @profiling = options[:profile] ? true : false
      env = parse_context&.environment || environment || options[:environment] || @environment
      source = source.to_s.to_str
      unless source.valid_encoding?
        raise Liquid::TemplateEncodingError, "Invalid template encoding"
      end

      if partial_nesting_depth(source) > Liquid::Block::MAX_DEPTH
        raise Liquid::StackLevelError.new("Nesting too deep", line_number: 1)
      end

      line_numbers = !parse_context.line_number.nil? if parse_context
      line_numbers = options[:line_numbers] if options.key?(:line_numbers)
      line_numbers = true if @profiling && !options.key?(:line_numbers)
      effective_error_mode = (error_mode || options[:error_mode] || parse_context&.error_mode || env&.error_mode || :strict).to_sym
      source = normalize_bug_compatible_pretrim(source) if options[:bug_compatible_whitespace_trimming]
      source, doc_body_placeholders = protect_doc_block_bodies(source, line_numbers: line_numbers)
      source, compatibility_warnings, compatibility_render_errors = recover_non_strict_parse_compatibility(
        source,
        effective_error_mode,
        line_numbers: line_numbers
      )
      source, numeric_identifier_aliases = normalize_non_strict_numeric_assign_targets(
        source,
        effective_error_mode
      )
      source = normalize_cycle_compatibility(
        source,
        effective_error_mode,
        line_numbers: line_numbers,
        numeric_identifier_aliases: numeric_identifier_aliases
      )
      source = normalize_inline_comment_compatibility(source, line_numbers: line_numbers)
      source = normalize_raw_compatibility(source, line_numbers: line_numbers)
      source = normalize_tablerow_compatibility(source, effective_error_mode, line_numbers: line_numbers)
      source = normalize_non_strict_case_compatibility(source, effective_error_mode)
      source = restore_doc_block_bodies(source, doc_body_placeholders)
      rewritten_source, custom_tags, custom_tag_registry_name = rewrite_custom_tags(
        source,
        env,
        line_numbers: line_numbers,
        error_mode: effective_error_mode,
        parse_options: options
      )
      normalized_source = normalize_source(rewritten_source, effective_error_mode)
      prime_expression_cache!(normalized_source, parse_context, effective_error_mode, options)
      @line_numbers_enabled = line_numbers
      @source = source
      @custom_tags = custom_tags
      @custom_tag_registry_name = custom_tag_registry_name
      @custom_tag_registry = CustomTagRegistryDrop.new(self, custom_tags)
      @compatibility_render_errors = compatibility_render_errors
      @handle = Liquid::RustExtension.ext_parse(normalized_source, line_numbers, effective_error_mode.to_s, env&.native_handle)
      @errors = Array(Liquid::RustExtension.ext_template_errors(@handle))
      @warnings = Array(Liquid::RustExtension.ext_template_warnings(@handle)) + compatibility_warnings
      root_locale = parse_context&.locale || options[:locale] || Liquid::I18n.new
      @root = Liquid::AstTemplateRoot.build(
        source,
        environment: env,
        error_mode: effective_error_mode,
        line_numbers: line_numbers,
        locale: root_locale
      )
      self
    rescue ::StandardError => error
      raise error if error.is_a?(Liquid::TemplateEncodingError) || error.is_a?(Liquid::StackLevelError)

      raise Liquid::SyntaxError.wrap(
        error,
        default_class: Liquid::SyntaxError,
        line_numbers: line_numbers,
        source: source,
        error_mode: effective_error_mode
      )
    end

    def render(*args)
      render_args = args.dup
      options = extract_render_options(render_args)
      context = build_render_context(render_args)
      apply_options_to_context(context, options)
      with_render_mode(context, strict: false) do
        profiling_started_at = profiling_now if @profiling

        if (preflight_error = preflight_literal_partial_error(context))
          @errors = [preflight_error]
          rendered = preflight_error.to_s
          record_profile(context, profiling_started_at) if @profiling
          return options[:output] ? options[:output] << rendered : rendered
        end

        context.resource_limits.reset

        begin
          rendered = with_custom_tag_registry(context) do
            rendered = render_with_native_retry(context, strict: false)
            render_custom_tags(rendered, context)
          end
          original_native_errors = Array(Liquid::RustExtension.ext_template_errors(@handle))
          native_errors = recover_native_errors_from_context(context, original_native_errors)
          normalized_errors = normalize_template_errors(native_errors)
          apply_literal_partial_metadata!(context, normalized_errors)
          @errors = (normalized_errors + normalize_template_errors(context.errors) + duplicate_compatibility_render_errors).uniq
          raise_non_standard_errors!
          effective_renderer = effective_exception_renderer(context, options)
          rendered =
            if effective_renderer
              rewrite_rendered_errors_with_handler(
                rendered,
                original_native_errors,
                normalized_errors,
                effective_renderer
              )
            else
              rewrite_rendered_errors(rendered, original_native_errors, normalized_errors)
            end
          rendered = collapse_duplicate_error_prefixes(rendered)
          apply_literal_partial_metadata!(context, @errors)
          rendered = finalize_render_output(rendered, context)
          record_profile(context, profiling_started_at) if @profiling
          options[:output] ? options[:output] << rendered : rendered
        rescue ExceptionRendererRaised => error
          @errors = merged_template_errors(Array(Liquid::RustExtension.ext_template_errors(@handle)), context)
          raise error.error
        rescue ::StandardError => error
          wrapped = wrap_render_error(error)
          raise wrapped unless wrapped.is_a?(::StandardError)

          context.errors << wrapped if wrapped.is_a?(Liquid::Error)
          @errors = merged_template_errors([], context)
          rendered = collapse_duplicate_error_prefixes(wrapped.to_s)
          options[:output] ? options[:output] << rendered : rendered
        end
      end
    end

    def render!(*args)
      render_args = args.dup
      options = extract_render_options(render_args)
      context = build_render_context(render_args)
      apply_options_to_context(context, options)
      with_render_mode(context, strict: true) do
        profiling_started_at = profiling_now if @profiling

        if (preflight_error = preflight_literal_partial_error(context))
          @errors = [preflight_error]
          raise preflight_error
        end

        context.resource_limits.reset

        begin
          rendered = with_custom_tag_registry(context) do
            rendered = render_with_native_retry(context, strict: true)
            render_custom_tags(rendered, context)
          end
          rendered = context.apply_global_filter(rendered)
          @errors = merged_template_errors(Array(Liquid::RustExtension.ext_template_errors(@handle)), context)
          raise_non_standard_errors!
          record_profile(context, profiling_started_at) if @profiling
          options[:output] ? options[:output] << rendered : rendered
        rescue ::StandardError => error
          raise error unless liquid_error?(error)
          if (original = replay_context_lookup_exception(context, error.message))
            raise original
          end
          wrapped = wrap_render_error(error)
          if wrapped.is_a?(Liquid::Error) &&
              (wrapped.message.include?("Expected id but found end_of_string") || wrapped.message.include?("Unexpected character"))
            raise Liquid::SyntaxError.new(
              wrapped.message.sub(/\ALiquid error:\s*/, "").sub(/\ALiquid syntax error:\s*/, ""),
              line_number: wrapped.line_number,
              template_name: wrapped.template_name,
              cause: wrapped.cause
            )
          end

          raise wrapped
        end
      end
    end

    def render_to_output_buffer(context, output)
      if context.render_bang_mode?
        render!(context, output: output)
      else
        render(context, output: output)
      end
    end

    def render_fragment_to_output_buffer(context, output)
      if context.render_bang_mode?
        rendered = with_custom_tag_registry(context) do
          rendered = render_with_native_retry(context, strict: true)
          render_custom_tags(rendered, context)
        end
        @errors = merged_template_errors(Array(Liquid::RustExtension.ext_template_errors(@handle)), context)
        raise_non_standard_errors!
        output << rendered
        output
      else
        rendered = with_custom_tag_registry(context) do
          rendered = render_with_native_retry(context, strict: false)
          render_custom_tags(rendered, context)
        end
        original_native_errors = Array(Liquid::RustExtension.ext_template_errors(@handle))
        native_errors = recover_native_errors_from_context(context, original_native_errors)
        normalized_errors = normalize_template_errors(native_errors)
        apply_literal_partial_metadata!(context, normalized_errors)
        context.errors.concat(normalized_errors)
        @errors = (normalized_errors + normalize_template_errors(context.errors) + duplicate_compatibility_render_errors).uniq
        raise_non_standard_errors!
        effective_renderer = effective_exception_renderer(context, {})
        rendered =
          if effective_renderer
            rewrite_rendered_errors_with_handler(
              rendered,
              original_native_errors,
              normalized_errors,
              effective_renderer
            )
          else
            rewrite_rendered_errors(rendered, original_native_errors, normalized_errors)
          end
        output << rendered
        output
      end
    rescue ExceptionRendererRaised => error
      @errors = normalize_template_errors(Array(Liquid::RustExtension.ext_template_errors(@handle)))
      raise error.error
    rescue ::StandardError => error
      raise if context.render_bang_mode?

      wrapped = wrap_render_error(error)
      raise wrapped unless wrapped.is_a?(::StandardError)

      context.errors << wrapped if wrapped.is_a?(Liquid::Error)
      @errors = merged_template_errors([], context)
      output << wrapped.to_s
      output
    end

    def blank?
      root.blank?
    end

    private

    CYCLE_NAMED_SYNTAX = /\A(#{QuotedFragment})\s*\:\s*(.*)/om
    RUST_IDENTIFIER = /\A[A-Za-z_][A-Za-z0-9_-]*\z/

    def recover_non_strict_parse_compatibility(source, error_mode, line_numbers:)
      return [source, [], []] unless [:lax, :warn].include?(error_mode)

      source, warnings = recover_assign_range_compatibility(source, error_mode, line_numbers: line_numbers)
      render_errors = []

      if error_mode == :lax
        rewritten = source.gsub(/\{%-?\s*if\b.*?=\!.*?-?%\}.*?\{%-?\s*endif\s*-?%\}/m) do
          render_errors << Liquid::ArgumentError.new("Unknown operator =!")
          "Liquid error: Unknown operator =!"
        end
        return [rewritten, warnings, render_errors]
      end

      rewritten = +""
      cursor = 0
      pattern = /\{%-?\s*if\s+([^%]*?)(-?%\})|\{\{\s*%%%+\s*\}\}|\{\{\s*[A-Za-z_][\w-]*\.\s*\}\}/m

      source.to_enum(:scan, pattern).each do
        match = Regexp.last_match
        rewritten << source[cursor...match.begin(0)]
        token = match[0]
        line_number = line_number_for_offset(source, match.begin(0), line_numbers)

        if token.start_with?("{%") || token.start_with?("{%-")
          condition = match[1].to_s.strip
          if condition == "~~~"
            warnings << Liquid::SyntaxError.new(
              'Unexpected character ~ in "~~~"',
              line_number: line_number
            )
            rewritten << token.sub(match[1], "false")
          elsif condition.include?("=!")
            warnings << Liquid::SyntaxError.new(
              %(Unexpected character = in "#{condition}"),
              line_number: line_number
            )
            rewritten << token.sub(match[1], "false")
          else
            rewritten << token
          end
        elsif token.match?(/\A\{\{\s*%%%+\s*\}\}\z/m)
          warnings << Liquid::SyntaxError.new(
            %(Unexpected character % in #{token.strip.inspect}),
            line_number: line_number
          )
        else
          warnings << Liquid::SyntaxError.new(
            %(Expected id but found end_of_string in #{token.strip.inspect}),
            line_number: line_number
          )
        end

        cursor = match.end(0)
      end

      rewritten << source[cursor..]
      [rewritten, warnings, render_errors]
    end

    def recover_assign_range_compatibility(source, error_mode, line_numbers:)
      return [source, []] unless [:lax, :warn].include?(error_mode)

      pattern = /(\{%-?\s*assign\s+[\w-]+\s*=\s*)\(([^%]*?\|[^%]*?)\)(\s*-?%\})/m
      rewritten = +""
      warnings = []
      cursor = 0
      matched = false

      source.to_enum(:scan, pattern).each do
        match = Regexp.last_match
        matched = true
        rewritten << source[cursor...match.begin(0)]
        line_number = line_number_for_offset(source, match.begin(0), line_numbers)

        if error_mode == :warn
          warnings << Liquid::SyntaxError.new(
            %(Expected dotdot but found pipe in "{{(#{match[2]}) }}"),
            line_number: line_number
          )
        end

        rewritten << "#{match[1]}nil#{match[3]}"
        cursor = match.end(0)
      end

      return [source, []] unless matched

      rewritten << source[cursor..]
      [rewritten, warnings]
    end

    def normalize_non_strict_numeric_assign_targets(source, error_mode)
      return [source, {}] if error_mode == :strict2

      aliases = {}
      rewritten = +""
      index = 0
      skip_mode = nil

      while (tag = next_tag_token(source, index))
        rewritten << source[index...tag[:start]]

        if skip_mode
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          index = tag[:finish]
          next
        end

        if tag[:name] == "raw" || tag[:name] == "comment"
          skip_mode = tag[:name]
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        rewritten <<
          if tag[:name] == "assign"
            normalize_non_strict_numeric_assign_tag(tag, aliases)
          else
            tag[:token]
          end

        index = tag[:finish]
      end

      rewritten << source[index..]
      [rewritten, aliases]
    end

    def normalize_non_strict_numeric_assign_tag(tag, aliases)
      markup = tag[:markup].to_s
      return tag[:token] unless markup =~ /\A(.*?)=(.*)\z/m

      target = Regexp.last_match(1).to_s.strip
      return tag[:token] unless target.match?(/\A\d+\z/)

      replacement = aliases[target] ||= "__liquid_non_strict_num_#{target}__"
      rebuild_tag_token(tag, "#{replacement} = #{Regexp.last_match(2).to_s.strip}")
    end

    def normalize_cycle_compatibility(source, error_mode, line_numbers:, numeric_identifier_aliases:)
      rewritten = +""
      cursor = 0
      skip_mode = nil
      cycle_index = 0

      while (tag = next_tag_token(source, cursor))
        rewritten << source[cursor...tag[:start]]

        if skip_mode
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          cursor = tag[:finish]
          next
        end

        if tag[:name] == "raw" || tag[:name] == "comment"
          skip_mode = tag[:name]
          rewritten << tag[:token]
          cursor = tag[:finish]
          next
        end

        unless tag[:name] == "cycle"
          rewritten << tag[:token]
          cursor = tag[:finish]
          next
        end

        line_number = line_number_for_offset(source, tag[:start], line_numbers)
        rewritten << normalize_cycle_tag_token(
          tag,
          error_mode,
          line_number: line_number,
          cycle_index: cycle_index,
          numeric_identifier_aliases: numeric_identifier_aliases
        )
        cycle_index += 1
        cursor = tag[:finish]
      end

      rewritten << source[cursor..]
      rewritten
    end

    def normalize_inline_comment_compatibility(source, line_numbers:)
      rewritten = +""
      index = 0
      skip_mode = nil

      while (tag = next_tag_token(source, index))
        rewritten << source[index...tag[:start]]

        if skip_mode
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          index = tag[:finish]
          next
        end

        if tag[:name] == "raw" || tag[:name] == "comment"
          skip_mode = tag[:name]
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        if tag[:name] == "#"
          validate_inline_comment_markup!(
            tag[:markup],
            line_number_for_offset(source, tag[:start], line_numbers)
          )
          rewritten.sub!(/[ \t\f\r\n]+\z/, "") if tag[:left_trim]
          index = tag[:finish]
          index = skip_trimmed_whitespace(source, index) if tag[:right_trim]
          next
        end

        if tag[:name] == "liquid"
          rewritten << expand_compat_liquid_tag(
            tag,
            line_number_for_offset(source, tag[:start], line_numbers)
          )
          index = tag[:finish]
        else
          rewritten << tag[:token]
          index = tag[:finish]
        end
      end

      rewritten << source[index..]
      rewritten
    end

    def normalize_raw_compatibility(source, line_numbers:)
      rewritten = +""
      index = 0
      skip_mode = nil

      while (tag = next_tag_token(source, index))
        rewritten << source[index...tag[:start]]

        if skip_mode
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          index = tag[:finish]
          next
        end

        if tag[:name] == "comment" || tag[:name] == "doc"
          skip_mode = tag[:name]
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        unless tag[:name] == "raw"
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        line_number = line_number_for_offset(source, tag[:start], line_numbers)
        validate_raw_markup!(tag[:markup], line_number)
        raw_block = extract_raw_block(source, tag, line_numbers: line_numbers)

        rewritten.sub!(/[ \t\f\r\n]+\z/, "") if tag[:left_trim]
        rewritten << "{% raw %}"
        rewritten << raw_block[:body_source]
        rewritten << "{% endraw %}"

        index = raw_block[:finish]
        index = skip_trimmed_whitespace(source, index) if raw_block[:close_tag][:right_trim]
      end

      rewritten << source[index..]
      rewritten
    end

    def extract_raw_block(source, opening_tag, line_numbers:)
      remainder = source[opening_tag[:finish]..].to_s
      search_index = 0

      while (terminator = remainder.match(/-?%\}/, search_index))
        candidate_end = terminator.end(0)
        candidate = remainder[0...candidate_end]

        if candidate =~ /\A(.*)(\{%-?\s*(\w+)\s*.*?-?%\})\z/m && Regexp.last_match(3) == "endraw"
          close_token = Regexp.last_match(2)
          close_start = opening_tag[:finish] + Regexp.last_match.begin(2)
          close_finish = opening_tag[:finish] + Regexp.last_match.end(2)

          return {
            body_source: source[opening_tag[:finish]...close_start],
            close_tag: {
              token: close_token,
              start: close_start,
              finish: close_finish,
              left_trim: close_token.start_with?("{%-"),
              right_trim: close_token.end_with?("-%}")
            },
            finish: close_finish
          }
        end

        search_index = candidate_end
      end

      raise Liquid::SyntaxError.new(
        "'raw' tag was never closed",
        line_number: line_number_for_offset(source, opening_tag[:start], line_numbers)
      )
    end

    def expand_compat_liquid_tag(tag, start_line)
      prefix_lines = compat_liquid_prefix_lines(tag)
      suffix_lines = compat_liquid_suffix_lines(tag)
      body_lines = expand_compat_liquid_lines(
        normalize_liquid_inline_comments(tag[:markup]).lines,
        start_line ? start_line + prefix_lines : nil,
        liquid_tag_closing_line(tag, start_line)
      )
      if body_lines.all?(&:empty?)
        body_lines = ["{%- echo '' -%}"] + Array.new([body_lines.length - 1, 0].max, "")
      end

      (Array.new(prefix_lines, "") + body_lines + Array.new(suffix_lines, "")).join("\n")
    end

    def normalize_liquid_inline_comments(markup)
      markup.to_s.lines.reject { |line| line.lstrip.start_with?("#") }.join
    end

    def expand_compat_liquid_lines(lines, start_line, closing_line)
      stack = []

      lines.each_with_index.map do |line, index|
        stripped = line.strip
        line_number = start_line ? start_line + index : nil

        if stripped.empty? || stripped == "liquid"
          ""
        elsif stripped.start_with?("liquid ")
          expand_compat_liquid_inline(stripped.delete_prefix("liquid").strip, line_number)
        else
          validate_compat_liquid_statement!(stripped, line_number, stack)
          "{%- #{stripped} -%}"
        end
      end.tap do
        next if stack.empty?

        tag_name, = stack.last
        raise Liquid::SyntaxError.new(
          "'#{tag_name}' tag was never closed",
          line_number: closing_line
        )
      end
    end

    def expand_compat_liquid_inline(markup, line_number)
      stripped = markup.to_s.strip
      return "" if stripped.empty? || stripped == "liquid"
      return expand_compat_liquid_inline(stripped.delete_prefix("liquid").strip, line_number) if stripped.start_with?("liquid ")

      stack = []
      validate_compat_liquid_statement!(stripped, line_number, stack)
      unless stack.empty?
        tag_name, opened_line = stack.last
        raise Liquid::SyntaxError.new(
          "'#{tag_name}' tag was never closed",
          line_number: opened_line
        )
      end

      "{%- #{stripped} -%}"
    end

    def validate_compat_liquid_statement!(markup, line_number, stack)
      unless markup.to_s.match?(/\A#{Liquid::TagName}(?:\s+.*)?\z/o)
        raise Liquid::SyntaxError.new(
          "Unknown tag '#{markup}'",
          line_number: line_number
        )
      end

      tag_name = markup.to_s.split(/\s+/, 2).first

      case tag_name
      when "if", "unless", "for", "case", "capture", "comment", "raw", "tablerow", "doc"
        stack << [tag_name, line_number]
      when "endif", "endunless", "endfor", "endcase", "endcapture", "endcomment", "endraw", "endtablerow", "enddoc"
        expected = tag_name.delete_prefix("end")
        if stack.empty? || stack.last[0] != expected
          raise Liquid::SyntaxError.new(
            "'#{tag_name}' is not a valid delimiter for liquid tags. use %}",
            line_number: line_number
          )
        end
        stack.pop
      end
    end

    def liquid_tag_line_offset(tag)
      tag[:token].to_s.lines.first.to_s.include?("liquid") ? 0 : 1
    end

    def liquid_tag_closing_line(tag, start_line)
      return nil unless start_line

      start_line + tag[:token].to_s.count("\n")
    end

    def compat_liquid_prefix_lines(tag)
      token_lines = tag[:token].to_s.lines
      first_line = token_lines.first.to_s

      if first_line.include?("liquid")
        first_line.sub(/\A.*?\bliquid\b/, "").strip.empty? && token_lines.length > 1 ? 1 : 0
      else
        2
      end
    end

    def compat_liquid_suffix_lines(tag)
      tag[:token].to_s.lines.length > 1 ? 1 : 0
    end

    def validate_inline_comment_markup!(markup, line_number)
      return unless markup.to_s.match?(/\n\s*[^#\s]/)

      raise Liquid::SyntaxError.new(
        "Syntax error in tag '#' - Each line of comments must be prefixed by the '#' character",
        line_number: line_number
      )
    end

    def validate_raw_markup!(markup, line_number)
      return if markup.to_s.empty?

      raise Liquid::SyntaxError.new(
        "Syntax Error in 'raw' - Valid syntax: raw",
        line_number: line_number
      )
    end

    def normalize_tablerow_compatibility(source, error_mode, line_numbers:)
      rewritten = +""
      index = 0
      skip_mode = nil

      while (tag = next_tag_token(source, index))
        rewritten << source[index...tag[:start]]

        if skip_mode
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          index = tag[:finish]
          next
        end

        if tag[:name] == "raw" || tag[:name] == "comment"
          skip_mode = tag[:name]
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        unless tag[:name] == "tablerow"
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        line_number = line_number_for_offset(source, tag[:start], line_numbers)
        rewritten <<
          if error_mode == :strict2
            validate_strict2_tablerow_markup!(tag[:markup], line_number)
            tag[:token]
          else
            normalize_non_strict_tablerow_tag(tag, line_number)
          end

        index = tag[:finish]
      end

      rewritten << source[index..]
      rewritten
    end

    def normalize_non_strict_tablerow_tag(tag, line_number)
      normalized_markup = normalize_non_strict_tablerow_markup(tag[:markup], line_number)
      rebuild_tag_token(tag, normalized_markup)
    end

    def normalize_non_strict_tablerow_markup(markup, line_number)
      stripped = normalize_lax_range_markup(markup.to_s.strip)
      syntax = /\A(\w+)\s+in\s+(#{QuotedFragment}+)(.*)\z/o
      return stripped unless (match = stripped.match(syntax))

      variable_name = match[1]
      collection_markup = normalize_non_strict_tablerow_expression(match[2], line_number)
      attributes = normalize_non_strict_tablerow_attributes(match[3], line_number)
      [variable_name, "in", collection_markup, attributes].reject(&:empty?).join(" ")
    end

    def normalize_non_strict_tablerow_attributes(markup, line_number)
      allowed = %w[cols limit offset range]

      markup.to_s.scan(TagAttributes).filter_map do |key, value|
        next unless allowed.include?(key)

        normalized = normalize_non_strict_tablerow_expression(value, line_number)
        next unless normalized

        "#{key}: #{normalized}"
      end.join(", ")
    end

    def normalize_non_strict_tablerow_expression(markup, line_number)
      source = normalize_lax_range_markup(markup.to_s.strip)
      return source if source.empty?

      parse_context = Liquid::ParseContext.new(
        environment: @environment,
        line_number: line_number,
        error_mode: :strict
      )
      parsed = parse_context.parse_expression(source, safe: true)
      serialize_cycle_expression(parsed)
    rescue Liquid::Error
      "nil"
    end

    def validate_strict2_tablerow_markup!(markup, line_number)
      parse_context = Liquid::ParseContext.new(
        environment: @environment,
        line_number: line_number,
        error_mode: :strict2
      )
      parser = parse_context.new_parser(markup)
      allowed = %w[cols limit offset range]

      parser.consume(:id)
      unless parser.id?("in")
        raise Liquid::SyntaxError.new(
          %(For loops require an 'in' clause in #{markup.inspect}),
          line_number: line_number
        )
      end

      parse_context.safe_parse_expression(parser)
      parser.consume?(:comma)

      while parser.look(:id)
        key = parser.consume
        unless allowed.include?(key)
          raise Liquid::SyntaxError.new(
            %(Invalid attribute '#{key}' in tablerow loop. Valid attributes are cols, limit, offset, and range in #{markup.inspect}),
            line_number: line_number
          )
        end

        parser.consume(:colon)
        parse_context.safe_parse_expression(parser)
        parser.consume?(:comma)
      end

      parser.consume(:end_of_string)
    end

    def normalize_non_strict_case_compatibility(source, error_mode)
      return source if error_mode == :strict2

      rewritten = +""
      index = 0
      skip_mode = nil

      while (tag = next_tag_token(source, index))
        rewritten << source[index...tag[:start]]

        if skip_mode
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          index = tag[:finish]
          next
        end

        if tag[:name] == "raw" || tag[:name] == "comment" || tag[:name] == "doc"
          skip_mode = tag[:name]
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        normalized =
          case tag[:name]
          when "case"
            normalize_non_strict_case_expression(tag[:markup])
          when "when"
            normalize_non_strict_case_when_markup(tag[:markup])
          else
            nil
          end

        rewritten << (normalized ? rebuild_tag_token(tag, normalized) : tag[:token])
        index = tag[:finish]
      end

      rewritten << source[index..]
      rewritten
    end

    def normalize_non_strict_case_when_markup(markup)
      expressions, separator =
        if markup.include?(" or ")
          [markup.split(/\s+or\s+/), " or "]
        else
          [markup.split(/\s*,\s*/), ", "]
        end

      expressions
        .map { |part| normalize_non_strict_case_expression(part) }
        .reject(&:empty?)
        .join(separator)
    end

    def normalize_non_strict_case_expression(markup)
      normalized = markup.to_s.gsub(/([A-Za-z_][\w-]*)\s*=>\s*([A-Za-z_][\w-]*)/, '\1.\2')
      normalize_lax_expression(normalized, {})
    end

    def normalize_cycle_tag_token(tag, error_mode, line_number:, cycle_index:, numeric_identifier_aliases:)
      if error_mode == :strict2
        validate_cycle_strict2_markup!(tag[:markup], line_number)
        return tag[:token]
      end

      normalized_markup = normalize_lax_cycle_markup(
        tag[:markup],
        cycle_index,
        numeric_identifier_aliases
      )
      return tag[:token] unless normalized_markup

      opening = tag[:left_trim] ? "{%-" : "{%"
      closing = tag[:right_trim] ? "-%}" : "%}"
      "#{opening} cycle #{normalized_markup} #{closing}"
    end

    def validate_cycle_strict2_markup!(markup, line_number)
      parse_context = Liquid::ParseContext.new(
        environment: @environment,
        line_number: line_number,
        error_mode: :strict2
      )
      parser = parse_context.new_parser(markup)

      raise Liquid::SyntaxError.new(parse_context.locale.t("errors.syntax.cycle"), line_number: line_number) if parser.look(:end_of_string)

      first_expression = parse_context.safe_parse_expression(parser)
      if parser.look(:colon)
        parser.consume(:colon)
        parse_context.safe_parse_expression(parser)
      else
        first_expression
      end

      while parser.consume?(:comma)
        break if parser.look(:end_of_string)

        parse_context.safe_parse_expression(parser)
      end

      parser.consume(:end_of_string)
    rescue Liquid::SyntaxError => error
      error.line_number ||= line_number
      raise
    end

    def normalize_lax_cycle_markup(markup, cycle_index, numeric_identifier_aliases)
      stripped = markup.to_s.strip
      return nil if stripped.empty?

      if (named_match = stripped.match(CYCLE_NAMED_SYNTAX))
        name = normalize_cycle_expression_fragment(named_match[1], numeric_identifier_aliases)
        values = cycle_markup_values(named_match[2], numeric_identifier_aliases)
        return nil if name.nil? || values.empty?

        return "#{name}: #{values.map { |entry| entry[:markup] }.join(', ')}"
      end

      return nil unless stripped.match?(/\A#{QuotedFragment}+/o)

      values = cycle_markup_values(stripped, numeric_identifier_aliases)
      return nil if values.empty?

      if values.any? { |entry| entry[:lookup] }
        synthetic_name = "__liquid_cycle_#{cycle_index}__".inspect
        "#{synthetic_name}: #{values.map { |entry| entry[:markup] }.join(', ')}"
      else
        values.map { |entry| entry[:markup] }.join(", ")
      end
    end

    def cycle_markup_values(markup, numeric_identifier_aliases)
      markup.split(",").filter_map do |segment|
        match = segment.match(/\s*(#{QuotedFragment})\s*/o)
        next unless match

        parsed = Liquid::Expression.parse(match[1])
        normalized = serialize_cycle_expression(parsed, numeric_identifier_aliases)
        next unless normalized

        {
          markup: normalized,
          lookup: parsed.is_a?(Liquid::VariableLookup),
        }
      end
    end

    def normalize_cycle_expression_fragment(fragment, numeric_identifier_aliases)
      serialize_cycle_expression(Liquid::Expression.parse(fragment), numeric_identifier_aliases)
    end

    def serialize_cycle_expression(value, numeric_identifier_aliases = {})
      case value
      when Liquid::VariableLookup
        serialize_cycle_variable_lookup(value, numeric_identifier_aliases)
      when Liquid::RangeLookup
        "(#{serialize_cycle_expression(value.start_obj, numeric_identifier_aliases)}..#{serialize_cycle_expression(value.end_obj, numeric_identifier_aliases)})"
      when Range
        "(#{serialize_cycle_expression(value.begin, numeric_identifier_aliases)}..#{serialize_cycle_expression(value.end, numeric_identifier_aliases)})"
      when String
        value.inspect
      when Integer, Float
        value.to_s
      when TrueClass
        "true"
      when FalseClass
        "false"
      when NilClass
        "nil"
      else
        value.to_s.inspect
      end
    end

    def serialize_cycle_variable_lookup(lookup, numeric_identifier_aliases)
      markup = serialize_cycle_lookup_root(lookup.name, numeric_identifier_aliases)
      Array(lookup.lookups).each do |segment|
        if segment.is_a?(String) && segment.match?(RUST_IDENTIFIER)
          markup << ".#{segment}"
        else
          markup << "[#{serialize_cycle_expression(segment, numeric_identifier_aliases)}]"
        end
      end
      markup
    end

    def serialize_cycle_lookup_root(root, numeric_identifier_aliases)
      if root.is_a?(String) && numeric_identifier_aliases.key?(root)
        return numeric_identifier_aliases[root]
      end

      if root.is_a?(String) && root.match?(RUST_IDENTIFIER)
        root
      else
        "[#{serialize_cycle_expression(root, numeric_identifier_aliases)}]"
      end
    end

    def line_number_for_offset(source, offset, enabled)
      return nil unless enabled

      source.byteslice(0, offset).count("\n") + 1
    end

    def prime_expression_cache!(source, parse_context, error_mode, options = {})
      context =
        parse_context || Liquid::ParseContext.new(
          options.merge(
          environment: @environment,
          line_number: @line_numbers_enabled ? 1 : nil,
          error_mode: error_mode
          )
        )
      scanner = StringScanner.new("")

      source.to_s.to_enum(:scan, /\{\{(.*?)\}\}/m).each do
        match = Regexp.last_match
        markup = expression_cache_markup(match[1])
        next if markup.empty?

        parse_expression_cache_markup!(context, markup, scanner, match.begin(0), source)
      end

      source.to_s.to_enum(:scan, /\{%-?\s*assign\s+[^\n=]+=\s*(.*?)\s*-?%\}/m).each do
        match = Regexp.last_match
        markup = expression_cache_markup(match[1])
        next if markup.empty?

        parse_expression_cache_markup!(context, markup, scanner, match.begin(0), source)
      end
    end

    def expression_cache_markup(markup)
      markup.to_s.split("|", 2).first.to_s.strip
    end

    def parse_expression_cache_markup!(context, markup, scanner, offset, source)
      Liquid::Expression.parse(markup, scanner, context.instance_variable_get(:@expression_cache))
    rescue Liquid::SyntaxError => error
      raise Liquid::SyntaxError.new(
        error_message_body(error),
        line_number: line_number_for_offset(source, offset, true) || 1,
        template_name: error.template_name,
        markup_context: error.markup_context,
        cause: error.cause
      )
    end

    def literal_render_partial_reference
      return unless @source

      match = @source.match(/\{%\s*(render)\s+["']([^"']+)["'](?:\s+[^%]*)?%\}/)
      return unless match

      { tag: match[1], name: match[2] }
    end

    def literal_partial_error_reference
      return unless @source

      match = @source.match(/\{%\s*(include|render)\s+["']([^"']+)["'](?:\s+[^%]*)?%\}/)
      return unless match

      { tag: match[1], name: match[2] }
    end

    def preflight_literal_partial_error(context)
      ref = literal_render_partial_reference
      return unless ref

      partial_name = ref[:name]
      resolved_name = resolved_partial_template_name(context, partial_name)
      source =
        begin
          load_literal_partial_source(context, partial_name)
        rescue ::StandardError
          return Liquid::InternalError.new("internal", line_number: 1, template_name: name)
        end

      if (line_number = unterminated_output_line(source))
        return Liquid::SyntaxError.new(
          "Variable '{{' was not properly terminated with regexp: /\\}\\}/",
          line_number: line_number,
          template_name: resolved_name
        )
      end

      if partial_nesting_depth(source) > Liquid::Block::MAX_DEPTH
        return Liquid::StackLevelError.new(
          "Nesting too deep",
          line_number: 1,
          template_name: resolved_name
        )
      end

      nil
    end

    def render_with_native_retry(context, strict:)
      attempts = 0

      begin
        if strict
          Liquid::RustExtension.ext_render_strict(@handle, context.native_handle)
        else
          Liquid::RustExtension.ext_render(@handle, context.native_handle)
        end
      rescue ::RuntimeError => error
        attempts += 1
        raise unless transient_native_render_error?(error) && attempts < 2

        retry
      end
    end

    def transient_native_render_error?(error)
      ["to_liquid", "length"].include?(error.message.to_s)
    end

    def apply_literal_partial_metadata!(context, errors)
      ref = literal_partial_error_reference
      return unless ref

      resolved_name = resolved_partial_template_name(context, ref[:name])
      Array(errors).map! do |error|
        next error unless error.is_a?(Liquid::Error)
        next error if error.template_name

        error.class.new(
          error_message_body(error),
          line_number: error.line_number || 1,
          template_name: resolved_name,
          markup_context: error.markup_context,
          cause: error.cause
        )
      end
    end

    def load_literal_partial_source(context, partial_name)
      cache = context.registers.static[:literal_partial_source_cache] ||= {}
      return cache[partial_name] if cache.key?(partial_name)

      cache[partial_name] = context.registers[:file_system].read_template_file(partial_name)
    end

    def resolved_partial_template_name(context, partial_name)
      factory = context.registers[:template_factory]
      return partial_name unless factory.respond_to?(:for)

      template = factory.for(partial_name)
      template.respond_to?(:name) && template.name ? template.name : partial_name
    rescue ::StandardError
      partial_name
    end

    def unterminated_output_line(source)
      source.to_s.lines.each_with_index do |line, index|
        return index + 1 if line.include?("{{") && !line.include?("}}")
      end

      nil
    end

    def partial_nesting_depth(source)
      depth = 0
      max_depth = 0
      source.to_s.scan(/\{%-?\s*(end)?(if|unless|for|case|capture|comment|raw|tablerow|doc)\b[^%]*-?%\}/).each do |closing, _name|
        if closing
          depth -= 1 if depth.positive?
        else
          depth += 1
          max_depth = [max_depth, depth].max
        end
      end
      max_depth
    end

    def recover_native_errors_from_context(context, native_errors)
      Array(native_errors).filter_map do |error|
        message =
          if error.respond_to?(:message)
            error.message
          elsif error.respond_to?(:to_s)
            error.to_s
          end
        next error unless message

        recovered = replay_context_lookup_exception(context, message)
        case recovered
        when :suppress
          nil
        when nil
          error
        else
          recovered
        end
      end
    end

    def parse_unknown_index_error(message)
      lines = message.to_s.lines.map(&:strip)
      root_name = lines.find { |line| line.start_with?("variable=") }&.delete_prefix("variable=")
      requested_index = lines.find { |line| line.start_with?("requested index=") }&.delete_prefix("requested index=")
      [root_name, requested_index]
    end

    def parse_unknown_variable_error(message)
      lines = message.to_s.lines.map(&:strip)
      lines.find { |line| line.start_with?("requested variable=") }&.delete_prefix("requested variable=")
    end

    def raw_context_root_lookup(context, key)
      context.scopes.to_a.each do |scope|
        result = context_root_lookup_in_scope(scope, key)
        return result[:value] if result[:status] == :found
      end

      Array(context.environments).each do |scope|
        result = context_root_lookup_in_scope(scope, key)
        return result[:value] if result[:status] == :found
      end

      Array(context.static_environments).each do |scope|
        result = context_root_lookup_in_scope(scope, key)
        return result[:value] if result[:status] == :found
      end

      nil
    end

    def context_root_lookup_in_scope(scope, key)
      if scope.is_a?(Hash)
        return { status: :found, value: scope[key] } if scope.key?(key)
        return { status: :missing }
      end

      if scope.respond_to?(:key?) && scope.key?(key)
        return { status: :found, value: scope[key] }
      end

      if scope.respond_to?(:invoke_drop)
        return { status: :found, value: scope[key] }
      end

      { status: :missing }
    rescue Liquid::Error => error
      { status: :error, error: error }
    rescue ::StandardError => error
      { status: :error, error: error }
    end

    def replay_context_lookup_exception(context, message)
      root_name, requested_index = parse_unknown_index_error(message)
      if root_name && requested_index
        result = context_root_lookup(context, root_name)
        return nil unless result[:status] == :found

        value = result[:value]
        return nil unless value&.respond_to?(:[])

        value[requested_index]
        return :suppress
      end

      requested_variable = parse_unknown_variable_error(message)
      return nil unless requested_variable

      result = context_root_lookup(context, requested_variable)
      case result[:status]
      when :found
        :suppress
      when :error
        result[:error]
      else
        nil
      end
    rescue ::StandardError => error
      error
    end

    def context_root_lookup(context, key)
      context.scopes.to_a.each do |scope|
        result = context_root_lookup_in_scope(scope, key)
        return result unless result[:status] == :missing
      end

      Array(context.environments).each do |scope|
        result = context_root_lookup_in_scope(scope, key)
        return result unless result[:status] == :missing
      end

      Array(context.static_environments).each do |scope|
        result = context_root_lookup_in_scope(scope, key)
        return result unless result[:status] == :missing
      end

      { status: :missing }
    end

    def build_render_context(args)
      case args.first
      when Liquid::Context
        args.shift
      when Liquid::Drop
        drop = args.shift
        build_template_context(
          [drop, base_lookup_assigns]
        ).tap do |context|
          drop.context = context
        end
      when Hash
        build_template_context(
          [args.shift, base_lookup_assigns]
        )
      when nil
        build_template_context(
          [base_lookup_assigns]
        )
      else
        raise Liquid::ArgumentError, "Expected Hash, Liquid::Drop, Liquid::Context, or nil as parameter"
      end
    end

    def build_template_context(environments)
      Liquid::Context.build(
        environment: @environment,
        environments: environments,
        registers: @registers,
        resource_limits: @resource_limits
      ).tap do |context|
        context.native_handle["persistent_assigns"] = @instance_assigns
      end
    end

    def base_lookup_assigns
      MergedAssigns.new(@assigns, @instance_assigns)
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
      context.native_handle["registers"] = context.registers.to_h
      context.exception_renderer = options[:exception_renderer] if options.key?(:exception_renderer)
      context.native_handle["exception_renderer"] =
        native_exception_renderer(effective_exception_renderer(context, options))
      context.global_filter = options[:global_filter] if options.key?(:global_filter)
      context.strict_variables = options[:strict_variables] if options.key?(:strict_variables)
      context.strict_filters = options[:strict_filters] if options.key?(:strict_filters)
      context.template_name ||= name
    end

    def effective_exception_renderer(context, options)
      if options.key?(:exception_renderer)
        context.exception_renderer
      else
        context.exception_renderer || context.environment&.exception_renderer
      end
    end

    def normalize_source(source, error_mode)
      invalid_identifier_aliases = {}
      source = normalize_lax_tag_source(source, invalid_identifier_aliases) if error_mode == :lax
      source = normalize_non_strict_partial_tags(source) unless error_mode == :strict2

      source.gsub(/\{\{(.*?)\}\}/m) do
        markup = Regexp.last_match(1)
        blank_markup = normalize_blank_output_markup(markup)
        normalized_markup =
          if blank_markup
            blank_markup
          else
            normalize_output_markup(
              markup,
              error_mode,
              invalid_identifier_aliases: invalid_identifier_aliases
            )
          end
        "{{#{normalized_markup}}}"
      end
    end

    def normalize_blank_output_markup(markup)
      stripped = markup.to_s.strip
      return " '' " if stripped.empty?
      return "- '' -" if stripped == "-"

      nil
    end

    def rewrite_custom_tags(source, environment, line_numbers:, error_mode:, parse_options: {})
      tags = Array(environment&.tags).to_h
      custom_tag_classes = tags.each_with_object({}) do |(tag_name, klass), acc|
        next unless klass.is_a?(Class) && klass <= Liquid::Tag
        next if NATIVE_CONFORMANCE_TAGS.include?(tag_name.to_s)

        acc[tag_name.to_s] = klass
      end
      if parse_options.key?(:include_options_blacklist) && !custom_tag_classes.key?("include")
        custom_tag_classes["include"] = Liquid::Include
      end
      custom_tag_classes["comment"] ||= Liquid::Comment
      custom_tag_classes["doc"] ||= Liquid::Doc

      return [source, [], nil] if custom_tag_classes.empty?

      custom_tags = []
      rewritten = +""
      index = 0
      skip_mode = nil
      placeholder_nonce = SecureRandom.hex(12)
      registry_name = custom_tag_registry_name(placeholder_nonce)

      while index < source.length
        tag = next_tag_token(source, index)
        unless tag
          rewritten << source[index..]
          break
        end

        rewritten << source[index...tag[:start]]

        if skip_mode
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          index = tag[:finish]
          next
        end

        if tag[:name] == "raw" || (tag[:name] == "comment" && !custom_tag_classes.key?("comment"))
          skip_mode = tag[:name]
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        custom_tag = custom_tag_classes[tag[:name]]
        unless custom_tag
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        if custom_tag <= Liquid::Block
          if custom_tag == Liquid::Doc && tag[:token] !~ /\A\{%-?\s*doc\s*-?%\}\z/m
            raise Liquid::SyntaxError.new(
              "Syntax Error in 'doc' - Valid syntax: {% doc %}{% enddoc %}",
              line_number: line_number_for_offset(source, tag[:start], line_numbers)
            )
          end

          custom_block =
            if custom_tag == Liquid::Doc
              extract_doc_block(source, tag, line_numbers: line_numbers)
            elsif custom_tag == Liquid::Comment
              extract_comment_block(source, tag, line_numbers: line_numbers)
            else
              extract_custom_block(source, tag)
            end
          unless custom_block
            rewritten << tag[:token]
            index = tag[:finish]
            next
          end

          body_source = custom_block[:body_source]
          body_source = body_source.sub(/\A[ \t\f\r\n]+/, "") if tag[:right_trim]
          body_source = body_source.sub(/[ \t\f\r\n]+\z/, "") if custom_block[:close_tag][:left_trim]

          rewritten.sub!(/[ \t\f\r\n]+\z/, "") if tag[:left_trim]
          line_number = source.byteslice(0, tag[:start]).count("\n") + 1
          parsed_tag = build_custom_tag(
            custom_tag,
            tag_name: tag[:name],
            markup: tag[:markup],
            environment: environment,
            line_number: line_number,
            body_source: body_source,
            closing_token: custom_block[:close_tag][:token],
            line_numbers: line_numbers,
            parse_options: parse_options,
            error_mode: error_mode
          )

          if custom_tag == Liquid::Comment
            rewritten << compat_comment_placeholder_source(tag, custom_block, body_source)
          else
            method_name = "tag_#{custom_tags.length}"
            custom_tags << {
              method_name: method_name,
              tag: parsed_tag
            }
            rewritten << custom_tag_render_variable(registry_name, method_name)
          end

          index = custom_block[:finish]
          index = skip_trimmed_whitespace(source, index) if custom_block[:close_tag][:right_trim]
          next
        end

        rewritten.sub!(/[ \t\f\r\n]+\z/, "") if tag[:left_trim]
        method_name = "tag_#{custom_tags.length}"
        line_number = source.byteslice(0, tag[:start]).count("\n") + 1
        parsed_tag = build_custom_tag(
          custom_tag,
          tag_name: tag[:name],
          markup: tag[:markup],
          environment: environment,
          line_number: line_number,
          parse_options: parse_options,
          error_mode: error_mode
        )
        custom_tags << {
          method_name: method_name,
          tag: parsed_tag
        }
        rewritten << custom_tag_render_variable(registry_name, method_name)

        index = tag[:finish]
        index = skip_trimmed_whitespace(source, index) if tag[:right_trim]
      end

      [rewritten, custom_tags, registry_name]
    end

    def render_custom_tags(rendered, _context)
      rendered
    end

    def with_custom_tag_registry(context)
      return yield if @custom_tag_registry_name.nil? || @custom_tag_registry.nil?

      context.stack(
        Liquid::Context::HIDDEN_SCOPE_KEY => true,
        @custom_tag_registry_name => @custom_tag_registry
      ) do
        yield
      end
    end

    def render_custom_tag(custom_tag, context)
      buffer = +""
      custom_tag[:tag].render_to_output_buffer(context, buffer)
      buffer
    end

    def build_custom_tag(klass, tag_name:, markup:, environment:, line_number:, body_source: nil, closing_token: nil, line_numbers: false, parse_options: {}, error_mode: nil)
      effective_line_number = line_numbers ? line_number : nil
      parse_context = Liquid::ParseContext.new(
        parse_options.merge(
          environment: environment || @environment,
          line_number: effective_line_number,
          error_mode: error_mode,
          custom_block_body_only: !body_source.nil?
        )
      )
      tag = klass.new(tag_name, markup, parse_context)

      if klass <= Liquid::Block
        tokenizer_source =
          if klass == Liquid::Comment && body_source
            +"#{body_source}#{closing_token}"
          else
            body_source.to_s
          end
        tokenizer = CustomBlockTokenizer.new(
          source: tokenizer_source,
          string_scanner: StringScanner.new(""),
          line_numbers: line_numbers,
          line_number: effective_line_number && effective_line_number + tag_token_line_span(tag_name, markup),
          environment: environment || @environment,
          error_mode: parse_context.error_mode,
          block_tag_names: custom_block_tag_names(environment)
        )
        tag.parse(tokenizer)
      else
        tag.parse(nil)
      end

      tag
    end

    def normalize_output_markup(markup, error_mode, invalid_identifier_aliases: {})
      markup = normalize_lax_output_markup(markup, invalid_identifier_aliases) if error_mode == :lax

      normalized =
        transform_unquoted_markup(markup) do |segment|
          segment = segment.gsub(/(?<=\w|\]|\))\s*\.\s*(?=\w)/, ".")
          segment
        end

      case error_mode
      when :strict2, :lax
        normalized = normalized.gsub(/,\s*(?=\||\z)/, "")
        normalized = normalized.gsub(/:\s*(?=\||\z)/, "")
      end

      return normalized unless error_mode == :lax

      normalized = normalized.sub(/\A(\s*)\[\s*\[/, '\1[')
      opens = count_unquoted_char(normalized, "[")
      closes = count_unquoted_char(normalized, "]")
      normalized += "]" * (opens - closes) if opens > closes
      normalized
    end

    def normalize_lax_tag_source(source, invalid_identifier_aliases)
      rewritten = +""
      index = 0
      skip_mode = nil

      while (tag = next_tag_token(source, index))
        rewritten << source[index...tag[:start]]

        if skip_mode
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          index = tag[:finish]
          next
        end

        if tag[:name] == "raw" || tag[:name] == "comment"
          skip_mode = tag[:name]
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        rewritten <<
          if tag[:name] == "liquid"
            expand_lax_liquid_tag(tag, invalid_identifier_aliases)
          else
            rebuild_tag_token(
              tag,
              normalize_lax_tag_markup(tag, invalid_identifier_aliases)
            )
          end
        index = tag[:finish]
      end

      rewritten << source[index..]
      rewritten
    end

    def normalize_non_strict_partial_tags(source)
      rewritten = +""
      index = 0
      skip_mode = nil

      while (tag = next_tag_token(source, index))
        rewritten << source[index...tag[:start]]

        if skip_mode
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          index = tag[:finish]
          next
        end

        if tag[:name] == "raw" || tag[:name] == "comment"
          skip_mode = tag[:name]
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        if tag[:name] == "include" || tag[:name] == "render"
          rewritten << rebuild_tag_token(tag, normalize_non_strict_partial_markup(tag[:markup]))
        else
          rewritten << tag[:token]
        end
        index = tag[:finish]
      end

      rewritten << source[index..]
      rewritten
    end

    def normalize_non_strict_partial_markup(markup)
      markup = markup.to_s.dup
      markup.gsub!(/([^\s,]+)\s*=>\s*([^\s,]+)/, '\1')
      markup.gsub!(/\s+[!~]{3,}(?=\s|$)/, " ")
      markup.gsub!(/\A[!~]{3,}\s+/, "")
      markup.squeeze!(" ")
      markup.strip
    end

    def normalize_bug_compatible_pretrim(source)
      rewritten = +""
      index = 0
      skip_mode = nil

      while (tag = next_tag_token(source, index))
        segment = source[index...tag[:start]]

        if skip_mode
          rewritten << segment
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          index = tag[:finish]
          next
        end

        if tag[:left_trim]
          rewritten << apply_bug_compatible_pretrim_segment(segment)
          rewritten << remove_left_trim_marker(tag[:token])
        else
          rewritten << segment
          rewritten << tag[:token]
        end

        skip_mode = tag[:name] if tag[:name] == "raw" || tag[:name] == "comment"
        index = tag[:finish]
      end

      rewritten << source[index..]
      rewritten
    end

    def apply_bug_compatible_pretrim_segment(segment)
      source = segment.to_s
      last_variable = nil
      source.scan(/\{\{.*?\}\}/m) { last_variable = Regexp.last_match }
      return bug_compatible_pretrim_segment(source) unless last_variable

      prefix = source[0...last_variable.end(0)]
      suffix = source[last_variable.end(0)..]
      prefix + bug_compatible_pretrim_segment(suffix)
    end

    def bug_compatible_pretrim_segment(segment)
      source = segment.to_s
      trimmed = source.sub(/[ \t\f\r\n]+\z/, "")
      return trimmed unless trimmed.empty? && !source.empty?

      source.byteslice(0, 1).to_s
    end

    def remove_left_trim_marker(token)
      token.to_s.sub(/\A\{%-/, "{%")
    end

    def rebuild_tag_token(tag, markup)
      left = tag[:left_trim] ? "{%-" : "{%"
      right = tag[:right_trim] ? "-%}" : "%}"
      content = markup.to_s.empty? ? tag[:name].to_s : "#{tag[:name]} #{markup}"
      "#{left} #{content} #{right}"
    end

    def normalize_lax_tag_markup(tag, invalid_identifier_aliases)
      markup = tag[:markup].to_s

      case tag[:name]
      when "if", "unless"
        normalize_lax_condition_markup(markup, invalid_identifier_aliases)
      when "assign"
        normalize_lax_assign_markup(markup, invalid_identifier_aliases)
      when "for"
        normalize_lax_range_markup(markup)
      when "liquid"
        normalize_lax_liquid_markup(markup, invalid_identifier_aliases)
      else
        normalize_lax_range_markup(markup)
      end
    end

    def normalize_lax_condition_markup(markup, invalid_identifier_aliases)
      transform_unquoted_markup(markup) do |segment|
        segment = segment.delete("()")
        segment = segment.sub(/\s*(?:&&|\|\|).*$/m, "")
        segment.split(/(\s+(?:and|or)\s+|\s*(?:==|!=|<>|>=|<=|>|<|contains)\s*)/).map do |part|
          if part.match?(/\A\s+(?:and|or)\s+\z/) ||
              part.match?(/\A\s*(?:==|!=|<>|>=|<=|>|<|contains)\s*\z/)
            part
          else
            normalize_lax_expression(part, invalid_identifier_aliases)
          end
        end.join
      end.strip
    end

    def normalize_lax_assign_markup(markup, invalid_identifier_aliases)
      return normalize_lax_range_markup(markup) unless markup =~ /\A(.*?)=(.*)\z/m

      lhs = normalize_lax_assign_target(Regexp.last_match(1).strip, invalid_identifier_aliases)
      rhs = normalize_lax_expression(Regexp.last_match(2), invalid_identifier_aliases)
      "#{lhs} = #{rhs}"
    end

    def normalize_lax_liquid_markup(markup, invalid_identifier_aliases)
      markup.each_line.map do |line|
        stripped = line.strip
        next stripped if stripped.empty?

        if stripped.start_with?("assign ")
          "assign #{normalize_lax_assign_markup(stripped.delete_prefix("assign "), invalid_identifier_aliases)}"
        else
          normalize_lax_range_markup(stripped)
        end
      end.join("\n")
    end

    def expand_lax_liquid_tag(tag, invalid_identifier_aliases)
      normalized_markup = normalize_lax_liquid_markup(tag[:markup], invalid_identifier_aliases)
      normalized_markup.each_line.each_with_object([]) do |line, expanded|
        stripped = line.strip
        next if stripped.empty?

        expanded << "{% #{stripped} %}"
      end.join
    end

    def normalize_lax_assign_target(target, invalid_identifier_aliases)
      if target.match?(/\A\d[\w-]*[A-Za-z_][\w-]*\z/)
        invalid_identifier_aliases[target] ||= "__liquid_lax_var_#{invalid_identifier_aliases.length}__"
      elsif target.match?(/\A\d+\z/)
        "__liquid_lax_ignored_#{target}__"
      else
        target
      end
    end

    def normalize_lax_output_markup(markup, invalid_identifier_aliases)
      stripped = markup.to_s.strip
      return "''" if stripped.match?(/\A-\s+.+\s+-\z/)

      segments = split_lax_pipes(markup)
      expression = segments.shift.to_s

      if expression.strip.empty? && !segments.empty?
        expression = segments.shift.to_s
      end

      expression = normalize_lax_expression(expression, invalid_identifier_aliases)

      broken_quote = false
      normalized_filters = segments.each_with_object([]) do |segment, filters|
        filter_markup, broken_quote = normalize_lax_filter_segment(segment, broken_quote)
        filters << filter_markup if filter_markup
      end

      ([expression] + normalized_filters.map { |filter| " #{filter}" }).join("|").strip
    end

    def normalize_lax_expression(markup, invalid_identifier_aliases)
      markup = normalize_lax_invalid_identifier(markup.to_s.strip, invalid_identifier_aliases)
      quirky_number = normalize_lax_numeric_expression(markup)
      return quirky_number if quirky_number

      markup = normalize_lax_range_markup(markup)

      if markup =~ /\A((?:true|false|nil|null|blank|empty|-?\d+(?:\.\d+)?|"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|[A-Za-z_][\w-]*(?:\[[^\]]*\]|\.[A-Za-z_][\w-]*)*|\([^)]+\)))(?:\s+.+)\z/m
        Regexp.last_match(1)
      else
        markup
      end
    end

    def normalize_lax_invalid_identifier(markup, invalid_identifier_aliases)
      invalid_identifier_aliases.each do |original, replacement|
        rewritten = markup.sub(/\A#{Regexp.escape(original)}(?=\z|\.|\[)/, replacement)
        return rewritten if rewritten != markup
      end

      markup
    end

    def normalize_lax_condition_operands(markup, invalid_identifier_aliases)
      markup.split(/(\s*(?:==|!=|<>|>=|<=|>|<|contains)\s*)/).each_with_index.map do |part, index|
        index.even? ? normalize_lax_expression(part, invalid_identifier_aliases) : part
      end.join
    end

    def normalize_lax_numeric_expression(markup)
      return unless markup.match?(/\A-?\d[\d.]*\z/)
      return unless markup.count(".") > 1

      parsed = Liquid::Expression.parse_number(markup, StringScanner.new(""))
      parsed.to_s if parsed
    end

    def normalize_lax_range_markup(markup)
      markup.to_s.gsub("...", "..")
    end

    def split_lax_pipes(markup)
      segments = []
      current = +""
      quote = nil
      escaped = false
      last_nonspace = nil

      markup.each_char do |char|
        if quote
          current << char
          if escaped
            escaped = false
          elsif char == "\\"
            escaped = true
          elsif char == quote
            quote = nil
          end
        elsif char == "|"
          segments << current
          current = +""
        elsif char == "'" || char == '"'
          if last_nonspace == char
            current << char
          else
            quote = char
            current << char
          end
        else
          current << char
        end

        last_nonspace = char unless char.match?(/\s/)
      end

      segments << current
      segments
    end

    def normalize_lax_filter_segment(segment, broken_quote)
      segment = segment.to_s.strip
      return [nil, broken_quote] if segment.empty?

      match = segment.match(/\A([A-Za-z_][\w-]*)/)
      return [nil, broken_quote] unless match

      filter_name = match[1]
      rest = segment[match[0].length..].to_s.lstrip
      return [filter_name, broken_quote] if rest.empty?

      colon_rest =
        if rest.start_with?(":")
          rest[1..]
        elsif rest =~ /\A[^A-Za-z0-9_\[\('""]+:\s*/
          rest.sub(/\A[^A-Za-z0-9_\[\('""]+:\s*/, "")
        else
          return [filter_name, broken_quote]
        end

      return [nil, broken_quote] if broken_quote

      argument, malformed_quote = extract_lax_filter_argument(colon_rest)
      return [filter_name, broken_quote] if argument.nil? || argument.empty?

      ["#{filter_name}:#{argument}", malformed_quote]
    end

    def extract_lax_filter_argument(source)
      source = source.to_s.lstrip
      return [nil, false] if source.empty?

      if source.start_with?('"', "'")
        quote = source[0]
        index = 1
        escaped = false

        while index < source.length
          char = source[index]
          if escaped
            escaped = false
          elsif char == "\\"
            escaped = true
          elsif char == quote
            argument = source[0..index]
            malformed_quote = source[index + 1] == quote
            return [argument, malformed_quote]
          end

          index += 1
        end

        return [source, true]
      end

      match = source.match(/\A([A-Za-z_][\w-]*(?:\[[^\]]*\]|\.[A-Za-z_][\w-]*)*|-?\d+(?:\.\d+)?|\([^)]+\)|true|false|nil|null|blank|empty)/)
      [match&.[](1), false]
    end

    def transform_unquoted_markup(markup)
      result = +""
      segment = +""
      quote = nil
      escaped = false

      markup.each_char do |char|
        if quote
          result << char
          if escaped
            escaped = false
          elsif char == "\\"
            escaped = true
          elsif char == quote
            quote = nil
          end
        elsif char == "'" || char == '"'
          result << yield(segment) unless segment.empty?
          segment.clear
          quote = char
          result << char
        else
          segment << char
        end
      end

      result << yield(segment) unless segment.empty?
      result
    end

    def next_tag_token(source, offset)
      start = source.index(/\{%\-?/, offset)
      return nil unless start

      left_trim = source[start + 2] == "-"
      content_start = start + (left_trim ? 3 : 2)
      inline_comment = source[content_start..].to_s.lstrip.start_with?("#")
      cursor = content_start
      quote = nil
      escaped = false

      while cursor < source.length
        char = source[cursor]

        if inline_comment
          if char == "-" && source[cursor + 1] == "%" && source[cursor + 2] == "}"
            finish = cursor + 3
            token = source[start...finish]
            return build_tag_token(start, finish, token, source[content_start...cursor], left_trim, true)
          elsif char == "%" && source[cursor + 1] == "}"
            finish = cursor + 2
            token = source[start...finish]
            return build_tag_token(start, finish, token, source[content_start...cursor], left_trim, false)
          end
        elsif quote
          if escaped
            escaped = false
          elsif char == "\\"
            escaped = true
          elsif char == quote
            quote = nil
          end
        else
          if char == "'" || char == '"'
            quote = char
          elsif char == "-" && source[cursor + 1] == "%" && source[cursor + 2] == "}"
            finish = cursor + 3
            token = source[start...finish]
            return build_tag_token(start, finish, token, source[content_start...cursor], left_trim, true)
          elsif char == "%" && source[cursor + 1] == "}"
            finish = cursor + 2
            token = source[start...finish]
            return build_tag_token(start, finish, token, source[content_start...cursor], left_trim, false)
          end
        end

        cursor += 1
      end

      nil
    end

    def extract_doc_block(source, opening_tag, line_numbers:)
      remainder = source[opening_tag[:finish]..].to_s
      depth = 1
      cursor = 0

      while (match = remainder.match(/\{%-?\s*(doc|enddoc)\b.*?-?%\}/m, cursor))
        tag_name = match[1]
        if tag_name == "doc"
          raise Liquid::SyntaxError.new(
            "Syntax Error in 'doc' - Nested doc tags are not allowed",
            line_number: line_number_for_offset(source, opening_tag[:start], line_numbers)
          )
        end

        depth -= 1
        if depth.zero?
          close_start = opening_tag[:finish] + match.begin(0)
          close_finish = opening_tag[:finish] + match.end(0)
          close_token = source[close_start...close_finish]

          return {
            body_source: source[opening_tag[:finish]...close_start],
            close_tag: {
              token: close_token,
              start: close_start,
              finish: close_finish,
              left_trim: close_token.start_with?("{%-"),
              right_trim: close_token.end_with?("-%}")
            },
            finish: close_finish
          }
        end

        cursor = match.end(0)
      end

      raise Liquid::SyntaxError.new(
        "'doc' tag was never closed",
        line_number: line_number_for_offset(source, opening_tag[:start], line_numbers)
      )
    end

    def protect_doc_block_bodies(source, line_numbers:)
      placeholder_nonce = SecureRandom.hex(12)
      doc_body_placeholders = []
      rewritten = +""
      index = 0
      skip_mode = nil

      while index < source.length
        tag = next_tag_token(source, index)
        unless tag
          rewritten << source[index..]
          break
        end

        rewritten << source[index...tag[:start]]

        if skip_mode
          rewritten << tag[:token]
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          index = tag[:finish]
          next
        end

        if tag[:name] == "raw" || tag[:name] == "comment"
          skip_mode = tag[:name]
          rewritten << tag[:token]
          index = tag[:finish]
          next
        end

        if tag[:name] == "doc" && tag[:token] =~ /\A\{%-?\s*doc\s*-?%\}\z/m
          doc_block = extract_doc_block(source, tag, line_numbers: line_numbers)
          placeholder = doc_block_body_placeholder(placeholder_nonce, doc_body_placeholders.length, doc_block[:body_source])
          doc_body_placeholders << {
            placeholder: placeholder,
            body_source: doc_block[:body_source]
          }
          rewritten << tag[:token] << placeholder << doc_block[:close_tag][:token]
          index = doc_block[:finish]
          next
        end

        rewritten << tag[:token]
        index = tag[:finish]
      end

      [rewritten, doc_body_placeholders]
    end

    def restore_doc_block_bodies(source, doc_body_placeholders)
      return source if doc_body_placeholders.empty?

      restored = source.dup
      doc_body_placeholders.each do |entry|
        restored.sub!(entry[:placeholder], entry[:body_source])
      end
      restored
    end

    def doc_block_body_placeholder(nonce, index, body_source)
      "__liquid_doc_body_#{nonce}_#{index}__#{("\n" * body_source.to_s.count("\n"))}"
    end

    def extract_comment_block(source, opening_tag, line_numbers:)
      remainder = source[opening_tag[:finish]..].to_s
      tokenizer = Liquid::Tokenizer.new(
        source: remainder,
        string_scanner: StringScanner.new(""),
        line_numbers: false
      )
      comment_tag_depth = 1
      consumed = 0

      while (token = tokenizer.shift)
        token_start = consumed
        consumed += token.length

        tag_name =
          if token.start_with?(BlockBody::TAGSTART) && token =~ BlockBody::FullToken
            Regexp.last_match(2)
          end

        case tag_name
        when "raw"
          consumed = consume_comment_raw_body!(tokenizer, consumed, line_numbers, source, opening_tag)
        when "comment"
          comment_tag_depth += 1
        when "endcomment"
          comment_tag_depth -= 1
          if comment_tag_depth.zero?
            close_start = opening_tag[:finish] + token_start
            close_finish = opening_tag[:finish] + consumed

            return {
              body_source: source[opening_tag[:finish]...close_start],
              close_tag: {
                token: token,
                start: close_start,
                finish: close_finish,
                left_trim: token.start_with?("{%-"),
                right_trim: token.end_with?("-%}")
              },
              finish: close_finish
            }
          end
        end
      end

      raise Liquid::SyntaxError.new(
        "'comment' tag was never closed",
        line_number: line_number_for_offset(source, opening_tag[:start], line_numbers)
      )
    end

    def build_tag_token(start, finish, token, raw_markup, left_trim, right_trim)
      if token =~ BlockBody::FullToken
        name = Regexp.last_match(2).to_s
        remainder = Regexp.last_match(4).to_s.strip
      else
        markup = raw_markup.to_s.strip
        if markup.start_with?("#")
          name = "#"
          remainder = markup[1..].to_s.strip
        else
          name, remainder = markup.split(/\s+/, 2)
        end
      end
      if name == "#"
        remainder = remainder.to_s.strip
      else
        remainder = remainder.to_s
      end
      {
        start: start,
        finish: finish,
        token: token,
        name: name.to_s,
        markup: remainder,
        left_trim: left_trim,
        right_trim: right_trim
      }
    end

    def compat_comment_placeholder_source(opening_tag, custom_block, body_source)
      preserved_newlines =
        opening_tag[:token].count("\n") +
        body_source.to_s.count("\n") +
        custom_block[:close_tag][:token].count("\n")

      "{% comment %}#{("\n" * preserved_newlines)}{% endcomment %}"
    end

    def extract_custom_block(source, opening_tag)
      cursor = opening_tag[:finish]
      depth = 1
      skip_mode = nil

      while (tag = next_tag_token(source, cursor))
        if skip_mode
          skip_mode = nil if tag[:name] == "end#{skip_mode}"
          cursor = tag[:finish]
          next
        end

        if tag[:name] == "raw" || tag[:name] == "comment"
          skip_mode = tag[:name]
          cursor = tag[:finish]
          next
        end

        if tag[:name] == opening_tag[:name]
          depth += 1
        elsif tag[:name] == "end#{opening_tag[:name]}"
          depth -= 1
          if depth.zero?
            return {
              body_source: source[opening_tag[:finish]...tag[:start]],
              close_tag: tag,
              finish: tag[:finish]
            }
          end
        end

        cursor = tag[:finish]
      end

      nil
    end

    def consume_comment_raw_body!(tokenizer, consumed, line_numbers, source, opening_tag)
      while (token = tokenizer.shift)
        consumed += token.length
        return consumed if token =~ BlockBody::FullTokenPossiblyInvalid && Regexp.last_match(2) == "endraw"
      end

      raise Liquid::SyntaxError.new(
        "'raw' tag was never closed",
        line_number: line_number_for_offset(source, opening_tag[:start], line_numbers)
      )
    end

    def skip_trimmed_whitespace(source, offset)
      cursor = offset
      cursor += 1 while cursor < source.length && source[cursor].match?(/[ \t\f\r\n]/)
      cursor
    end

    def custom_tag_registry_name(nonce)
      "__liquid_custom_tag_registry_#{nonce}"
    end

    def custom_tag_render_variable(registry_name, method_name)
      "{{ #{registry_name}.#{method_name} }}"
    end

    def tag_token_line_span(tag_name, markup)
      [tag_name, markup].compact.join(" ").count("\n")
    end

    class CustomBlockTokenizer < Liquid::Tokenizer
      DEFAULT_BLOCK_TAG_NAMES = %w[
        if unless case for capture comment raw liquid tablerow ifchanged doc
      ].freeze
      TAG_TOKEN = /\A\{%-?\s*(#{Liquid::TagName})\s*(.*?)\s*-?%\}\z/m
      VARIABLE_TOKEN = /\A\{\{-?\s*(.*?)\s*-?\}\}\z/m

      attr_reader :environment, :error_mode, :block_tag_names

      def initialize(environment:, error_mode:, block_tag_names:, **kwargs)
        @environment = environment
        @error_mode = error_mode
        @block_tag_names = (DEFAULT_BLOCK_TAG_NAMES + Array(block_tag_names).map(&:to_s)).uniq
        super(**kwargs)
      end

      def compile_template(source)
        Liquid::Template.parse(
          source,
          line_numbers: !line_number.nil?,
          error_mode: error_mode,
          environment: environment
        )
      end

      def parse_body_nodelist(source, line_number:)
        tokenizer = self.class.new(
          source: source,
          string_scanner: StringScanner.new(""),
          line_numbers: !line_number.nil?,
          line_number: line_number,
          environment: environment,
          error_mode: error_mode,
          block_tag_names: block_tag_names
        )
        parse_context = Liquid::ParseContext.new(
          environment: environment,
          line_number: line_number,
          error_mode: error_mode
        )
        nodes = []

        loop do
          token_line = tokenizer.line_number
          token = tokenizer.shift
          break unless token
          next if token.empty?

          parse_context.line_number = token_line

          if token.start_with?("{%")
            nodes << build_tag_node(tokenizer, parse_context, token)
          elsif token.start_with?("{{")
            nodes << build_variable_node(token, token_line)
          else
            nodes << token
          end
        end

        nodes
      end

      private

      def build_tag_node(tokenizer, parse_context, token)
        match = TAG_TOKEN.match(token)
        return token unless match

        tag_name = match[1]
        markup = match[2].to_s
        tag_class = environment&.tag_for_name(tag_name)
        return token unless tag_class

        tag_class.parse(tag_name, markup, tokenizer, parse_context)
      end

      def build_variable_node(token, token_line)
        match = VARIABLE_TOKEN.match(token)
        markup = match ? match[1].to_s : token
        parse_context = Liquid::ParseContext.new(
          environment: environment,
          line_number: token_line,
          error_mode: error_mode
        )
        Liquid::Variable.new(markup, parse_context)
      end
    end

    def custom_block_tag_names(environment)
      Array(environment&.tags).filter_map do |tag_name, klass|
        tag_name.to_s if klass.is_a?(Class) && klass <= Liquid::Block
      end
    end

    def count_unquoted_char(markup, target)
      count = 0
      transform_unquoted_markup(markup) do |segment|
        count += segment.count(target)
        segment
      end
      count
    end

    def liquid_error?(error)
      return true if error.is_a?(Liquid::Error)

      message = error.message.to_s
      message.start_with?("liquid:") || message.start_with?("Liquid error:")
    end

    def normalize_template_errors(errors)
      Array(errors).map do |error|
        current =
          if error.is_a?(Liquid::Error)
            duplicate_liquid_error(error)
          else
            ::RuntimeError.new(error.to_s)
          end

        wrap_render_error(current)
      end
    end

    def raise_non_standard_errors!
      exception = @errors.find { |error| error.is_a?(Exception) && !error.is_a?(::StandardError) }
      raise exception if exception
    end

    def rewrite_rendered_errors(rendered, native_errors, normalized_errors)
      native_errors.zip(normalized_errors).reduce(rendered.to_s) do |output, pair|
        native_error, normalized_error = pair
        next output unless native_error && normalized_error
        next output unless native_error.respond_to?(:to_s) && normalized_error.respond_to?(:to_s)
        normalized_error = normalized_error_for_rewrite(normalized_error)

        native_string =
          if native_error.is_a?(Liquid::Error)
            native_error.to_s
          elsif native_error.respond_to?(:message)
            prefixed_native_error_message(native_error) || Liquid::Error.wrap(native_error).to_s
          elsif native_error.to_s.start_with?("liquid:")
            Liquid::Error.wrap(::RuntimeError.new(native_error.to_s)).to_s
          else
            native_error.to_s
          end
        normalized_string = normalized_error.to_s
        next output if native_string.empty? || native_string == normalized_string

        rewrite_error_placeholder(output, native_string, normalized_string, normalized_error)
      end
    end

    def rewrite_rendered_errors_with_handler(rendered, native_errors, normalized_errors, handler)
      native_errors.zip(normalized_errors).reduce(rendered.to_s) do |output, pair|
        native_error, normalized_error = pair
        next output unless native_error && normalized_error
        normalized_error = normalized_error_for_rewrite(normalized_error)

        replacement =
          begin
            raise normalized_error
          rescue ::StandardError => current
            begin
              handler.call(current)
            rescue ::StandardError => raised
              raise ExceptionRendererRaised.new(raised.equal?(current) ? current : raised)
            end
          end
        rewrite_error_placeholder(output, native_error_string(native_error), replacement.to_s, normalized_error)
      end
    end

    def rewrite_error_placeholder(output, native_string, replacement, normalized_error)
      rewritten = replace_unique(output, native_string, replacement)
      return rewritten unless rewritten == output

      compat_placeholder = normalized_error_placeholder(normalized_error)
      return output if compat_placeholder.nil? || compat_placeholder.empty? || compat_placeholder == native_string

      replace_unique(output, compat_placeholder, replacement)
    end

    def native_error_string(native_error)
      if native_error.is_a?(Liquid::Error)
        native_error.to_s
      elsif native_error.respond_to?(:message)
        prefixed_native_error_message(native_error) || Liquid::Error.wrap(native_error).to_s
      elsif native_error.to_s.start_with?("liquid:")
        Liquid::Error.wrap(::RuntimeError.new(native_error.to_s)).to_s
      else
        native_error.to_s
      end
    end

    def prefixed_native_error_message(native_error)
      message = native_error.message.to_s
      return message if message.start_with?("Liquid error:", "Liquid syntax error:")

      nil
    end

    def normalized_error_placeholder(error)
      return unless error.is_a?(Liquid::Error)

      "#{error.class.prefix}: #{error_message_body(error)}"
    end

    def normalized_error_for_rewrite(error)
      return error unless error.is_a?(Liquid::Error)

      hydrate_render_error_metadata(duplicate_liquid_error(error))
    end

    def wrap_render_error(error)
      unless error.is_a?(Liquid::Error) || error.respond_to?(:cause)
        return Liquid::Error.wrap(::RuntimeError.new(error.to_s))
      end

      if error.is_a?(Liquid::Error) &&
          (error.message.include?("Expected id but found end_of_string") || error.message.include?("Unexpected character"))
        return Liquid::SyntaxError.new(
          error.message,
          line_number: error.line_number,
          template_name: error.template_name,
          cause: error.cause
        )
      end

      if error.is_a?(::RuntimeError) && !liquid_error?(error)
        return promote_runtime_error(error) || Liquid::Error.wrap(error)
      end

      if error.is_a?(Liquid::Error)
        promoted = promote_native_liquid_error(error)
        hydrated = hydrate_render_error_metadata(promoted || duplicate_liquid_error(error))
        return hydrated if promoted

        return hydrated
      end
      return error.cause if error.cause.is_a?(Liquid::Error)

      Liquid::Error.wrap(error)
    end

    def promote_native_liquid_error(error)
      return unless error.class == Liquid::Error

      message = error.message.to_s.sub(/\ALiquid error:\s*/, "")
      if message.include?("Expected id but found end_of_string") || message.include?("Unexpected character")
        return Liquid::SyntaxError.new(
          message,
          line_number: error.line_number,
          template_name: error.template_name,
          cause: error.cause
        )
      end
      wrapped = Liquid::Error.wrap(
        ::RuntimeError.new(message),
        default_class: Liquid::Error,
        line_numbers: @line_numbers_enabled,
        source: @source,
        error_mode: @environment&.error_mode
      )
      return wrapped unless wrapped.class == Liquid::Error

      promote_runtime_error(::RuntimeError.new(message), template_name: error.template_name)
    end

    def promote_runtime_error(error, template_name: nil)
      case error.message.to_s
      when "runtime error"
        Liquid::InternalError.new(
          "internal",
          line_number: infer_error_line_number(error.message),
          template_name: template_name,
          cause: error
        )
      when "exception"
        ::Exception.new(error.message)
      end
    end

    def finalize_render_output(rendered, context)
      context.apply_global_filter(rendered)
    end

    def with_render_mode(context, strict:)
      previous_mode = context.render_bang_mode?
      context.send(:render_bang_mode=, strict)
      yield
    ensure
      context.send(:render_bang_mode=, previous_mode)
    end

    def merged_template_errors(native_errors, context)
      (normalize_template_errors(native_errors) + normalize_template_errors(context.errors)).uniq
    end

    def profiling_now
      Process.clock_gettime(Process::CLOCK_MONOTONIC)
    end

    def record_profile(context, started_at)
      @profiler = context.profiler ||= Liquid::Profiler.new
      children = Liquid::Profiler::Builder.build(source: @source, context: context)
      @profiler.record_render(
        template_name: context.template_name,
        total_time: profiling_now - started_at,
        children: children
      )
    rescue ::StandardError
      nil
    end

    def duplicate_compatibility_render_errors
      Array(@compatibility_render_errors).map do |error|
        error.is_a?(Liquid::Error) ? duplicate_liquid_error(error) : error
      end
    end

    def hydrate_render_error_metadata(error)
      return error unless error.is_a?(Liquid::Error)

      if @line_numbers_enabled && error.line_number.nil?
        inferred_line = infer_error_line_number(error.message, error.cause&.message)
        error.line_number = inferred_line if inferred_line
        if error.line_number.nil? && error.is_a?(Liquid::ArgumentError) && error.message.include?("tag 'include'")
          error.line_number = 1
        end
      end

      if error.line_number.nil? && error.is_a?(Liquid::ArgumentError) && error_message_body(error) == "invalid integer"
        error.line_number = first_tag_line_number("tablerow", "for")
      end

      error
    end

    def duplicate_liquid_error(error)
      return error unless error.is_a?(Liquid::Error)
      return error if error.class == Liquid::Error

      error.class.new(
        error_message_body(error),
        line_number: error.line_number,
        template_name: error.template_name,
        markup_context: error.markup_context,
        cause: error.cause
      )
    end

    def error_message_body(error)
      return error.message.to_s unless error.class.respond_to?(:prefix)

      error.message.to_s.sub(/\A#{Regexp.escape(error.class.prefix)}(?: \([^)]+\))?:\s*/, "")
    end

    def infer_error_line_number(*messages)
      return nil unless @line_numbers_enabled && @source

      lookup_patterns(messages).each do |pattern|
        @source.each_line.with_index(1) do |line, line_number|
          return line_number if line.match?(pattern)
        end
      end

      nil
    end

    def first_tag_line_number(*tag_names)
      return nil unless @source

      pattern = /\{%-?\s*(#{tag_names.map { |tag| Regexp.escape(tag) }.join("|")})\b/
      @source.each_line.with_index(1) do |line, line_number|
        return line_number if line.match?(pattern)
      end

      nil
    end

    def lookup_patterns(messages)
      messages.compact.flat_map do |message|
        normalized = message.to_s.sub(/\ALiquid(?: syntax)? error(?: \([^)]+\))?:\s*/, "").strip
        next [] if normalized.empty?

        snake = normalized.tr(" ", "_")
        patterns = [
          Regexp.new(
            "(\\{\\{[^}]*\\b#{Regexp.escape(snake)}\\b[^}]*\\}\\}|\\{%-?[^%]*\\b#{Regexp.escape(snake)}\\b[^%]*-?%\\})"
          ),
        ]
        patterns << Regexp.new(Regexp.escape(normalized), Regexp::IGNORECASE)
        patterns
      end.compact.uniq
    end

    def replace_unique(text, from, to)
      first_index = text.index(from)
      return text unless first_index
      return text if text.index(from, first_index + from.length)

      text.dup.tap do |copy|
        copy[first_index, from.length] = to
      end
    end

    def native_exception_renderer(renderer)
      return nil unless renderer

      lambda do |error|
        begin
          raise wrap_render_error(error)
        rescue ::StandardError => current
          begin
            renderer.call(current).to_s
          rescue ::StandardError => raised
            raise raised if raised.equal?(current)

            raise ExceptionRendererRaised.new(raised)
          end
        end
      end
    end

    def collapse_duplicate_error_prefixes(text)
      text.to_s
        .gsub(/Liquid error:\s+(?:Liquid error:\s+)+/, "Liquid error: ")
        .gsub(/Liquid syntax error:\s+(?:Liquid syntax error:\s+)+/, "Liquid syntax error: ")
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
