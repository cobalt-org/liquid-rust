# frozen_string_literal: true

module Liquid
  class Context
    HIDDEN_SCOPE_KEY = :"__liquid_hidden_scope__"

    class ScopeView
      def initialize(context, scopes)
        @context = context
        @scopes = scopes
      end

      def size
        visible_scopes.size + @context.send(:live_scope_depth)
      end
      alias length size

      def [](index)
        live_depth = @context.send(:live_scope_depth)
        if index.is_a?(Integer) && index < live_depth
          {}
        else
          translated = index.is_a?(Integer) ? index - live_depth : index
          visible_scopes[translated]
        end
      end

      def first
        live_depth = @context.send(:live_scope_depth)
        live_depth.positive? ? {} : visible_scopes.first
      end

      def last
        visible_scopes.last
      end

      def to_a
        Array.new(@context.send(:live_scope_depth), {}) + visible_scopes
      end

      private

      def visible_scopes
        @scopes.reject { |scope| hidden_scope?(scope) }
      end

      def hidden_scope?(scope)
        scope.is_a?(Hash) && scope[Context::HIDDEN_SCOPE_KEY]
      end
    end

    attr_reader :errors, :registers, :environments, :resource_limits, :static_environments
    attr_accessor :exception_renderer, :template_name, :partial, :global_filter, :environment, :profiler

    def self.build(environment: Environment.default, environments: {}, outer_scope: {}, registers: {}, rethrow_errors: false, resource_limits: nil, static_environments: {}, parent: nil, &block)
      new(
        environments,
        outer_scope,
        registers,
        rethrow_errors,
        resource_limits,
        static_environments,
        environment,
        parent: parent,
        &block
      )
    end

    def initialize(
      environments = {},
      outer_scope = nil,
      registers = nil,
      rethrow_errors = false,
      resource_limits = nil,
      static_environments = nil,
      environment = nil,
      parent: nil,
      **legacy_options
    )
      recognized_legacy_keys = [:rethrow_errors, :registers, :environment, :resource_limits, :static_environments, :outer_scope]
      if legacy_options.any?
        if legacy_options.keys.none? { |key| recognized_legacy_keys.include?(key) }
          environments = legacy_options
        else
          rethrow_errors = legacy_options.fetch(:rethrow_errors, rethrow_errors)
          registers = legacy_options.fetch(:registers, registers)
          environment = legacy_options.fetch(:environment, environment)
          resource_limits = legacy_options.fetch(:resource_limits, resource_limits)
          static_environments = legacy_options.fetch(:static_environments, static_environments)
          outer_scope = legacy_options.fetch(:outer_scope, outer_scope)
        end
      end

      outer_scope ||= {}
      static_environments ||= {}
      environment ||= Environment.default
      registers ||= {}

      @environment = environment
      @environments = [environments].flatten
      @static_environments = [static_environments].flatten(1).freeze
      @scopes = [outer_scope || {}]
      @scope_view = ScopeView.new(self, @scopes)
      @registers = registers.is_a?(Liquid::Registers) ? registers : Liquid::Registers.new(registers)
      @errors = []
      @warnings = []
      @partial = false
      @strict_variables = false
      @strict_filters = false
      @resource_limits = resource_limits || ResourceLimits.new(environment.default_resource_limits)
      @base_scope_depth = 0
      @interrupts = []
      @filters = []
      @global_filter = nil
      @disabled_tags = {}
      @strainer = nil
      @string_scanner = StringScanner.new("")
      @render_bang_mode = false

      @registers.static[:cached_partials] ||= {}
      @registers.static[:file_system] ||= environment.file_system
      @registers.static[:template_factory] ||= Liquid::TemplateFactory.new
      @registers[:for] ||= {}
      @registers[:for_stack] ||= []

      self.exception_renderer = environment.exception_renderer
      self.exception_renderer = Liquid::RAISE_EXCEPTION_LAMBDA if rethrow_errors

      @native_handle = Liquid::RustExtension.ext_context_new(
        native_scope_chain,
        @registers.to_h,
        @environment.error_mode.to_s,
        parent&.native_handle
      )
      sync_native_metadata!

      yield self if block_given?

      squash_instance_assigns_with_environments
      sync_native_scopes!
    end

    def warnings
      @warnings
    end

    def scopes
      @scope_view
    end

    def strainer
      @strainer ||= @environment.create_strainer(self, @filters)
    end

    def add_filters(filters)
      @filters += [filters].flatten.compact
      @strainer = nil
      @native_handle["filters"] = @filters
      nil
    end

    def apply_global_filter(output)
      global_filter.nil? ? output : global_filter.call(output)
    end

    def interrupt?
      !@interrupts.empty?
    end

    def push_interrupt(interrupt)
      @interrupts.push(interrupt)
    end

    def pop_interrupt
      @interrupts.pop
    end

    def handle_error(error, line_number = nil)
      error = internal_error unless error.is_a?(Liquid::Error)
      error.template_name ||= template_name
      error.line_number ||= line_number
      errors.push(error)
      exception_renderer.call(error).to_s
    end

    def invoke(method, *args)
      result = strainer.invoke(method, *args)
      result.respond_to?(:to_liquid) ? result.to_liquid : result
    end

    def push(new_scope = {})
      @scopes.unshift(new_scope)
      check_overflow
      sync_native_scopes!
    end

    def merge(new_scopes)
      @scopes[0].merge!(new_scopes)
      sync_native_scopes!
    end

    def pop
      raise ContextError if @scopes.size == 1

      @scopes.shift
      sync_native_scopes!
    end

    def stack(new_scope = {})
      push(new_scope)
      yield
    ensure
      pop
    end

    def new_isolated_subcontext
      check_overflow

      self.class.build(
        environment: @environment,
        resource_limits: resource_limits,
        static_environments: static_environments,
        registers: Registers.new(registers),
        parent: self
      ).tap do |subcontext|
        subcontext.send(:base_scope_depth=, base_scope_depth + 1)
        subcontext.exception_renderer = exception_renderer
        subcontext.strict_variables = @strict_variables
        subcontext.strict_filters = @strict_filters
        subcontext.partial = partial
        subcontext.profiler = profiler
        subcontext.global_filter = global_filter
        subcontext.send(:render_bang_mode=, render_bang_mode?)
        subcontext.template_name = template_name
        subcontext.send(:filters=, @filters.dup)
        subcontext.send(:strainer=, nil)
        subcontext.send(:errors=, errors)
        subcontext.send(:warnings=, warnings)
        subcontext.send(:disabled_tags=, @disabled_tags.dup)
        subcontext.send(:sync_native_metadata!)
      end
    end

    def clear_instance_assigns
      @scopes[0] = {}
      sync_native_scopes!
    end

    def []=(key, value)
      @scopes[0][key] = value
      sync_native_scopes!
    end

    def [](expression)
      evaluate(Expression.parse(expression, @string_scanner))
    end

    def key?(key)
      find_variable(key, raise_on_not_found: false) != nil
    end

    def evaluate(object)
      object.respond_to?(:evaluate) ? object.evaluate(self) : object
    end

    def find_variable(key, raise_on_not_found: true)
      key = key.to_s

      variable =
        if Liquid::RustExtension.ext_context_has_live_root(@native_handle, key)
          wrap_live_root(key, Liquid::RustExtension.ext_context_find_live_root(@native_handle, key))
        else
          index = @scopes.find_index { |scope| scope.key?(key) }
          if index
            lookup_and_evaluate(@scopes[index], key, raise_on_not_found: raise_on_not_found)
          else
            try_variable_find_in_environments(key, raise_on_not_found: raise_on_not_found)
          end
        end

      if variable.nil? && @strict_variables && raise_on_not_found
        raise Liquid::UndefinedVariable, "undefined variable #{key}"
      end

      return nil if variable.nil?

      variable.context = self if variable.respond_to?(:context=)

      liquid_variable = variable.respond_to?(:to_liquid) ? variable.to_liquid : variable
      liquid_variable.context = self if variable != liquid_variable && liquid_variable.respond_to?(:context=)
      liquid_variable
    end

    def lookup_and_evaluate(obj, key, raise_on_not_found: true)
      if @strict_variables && raise_on_not_found && obj.respond_to?(:key?) && !obj.key?(key)
        raise Liquid::UndefinedVariable, "undefined variable #{key}"
      end

      value = obj[key]
      if value.is_a?(Proc) && obj.respond_to?(:[]=)
        obj[key] = value.arity == 0 ? value.call : value.call(self)
      else
        value
      end
    end

    def with_disabled_tags(tag_names)
      tag_names.each do |name|
        @disabled_tags[name] = @disabled_tags.fetch(name, 0) + 1
      end
      yield
    ensure
      tag_names.each do |name|
        @disabled_tags[name] -= 1
      end
    end

    def tag_disabled?(tag_name)
      @disabled_tags.fetch(tag_name, 0) > 0
    end

    def native_handle
      @native_handle
    end

    def render_bang_mode?
      @render_bang_mode
    end

    def strict_variables
      @strict_variables
    end

    def strict_variables=(value)
      @strict_variables = value
      @native_handle["strict_variables"] = value
    end

    def strict_filters
      @strict_filters
    end

    def strict_filters=(value)
      @strict_filters = value
      @native_handle["strict_filters"] = value
    end

    protected

    attr_writer :base_scope_depth, :warnings, :errors, :strainer, :filters, :disabled_tags, :render_bang_mode

    private

    attr_reader :base_scope_depth

    def lookup_value(value, key)
      if value.respond_to?(:context=)
        value.context = self
      end

      if value.respond_to?(:[])
        result = value[key]
      elsif value.respond_to?(:fetch) && key.is_a?(Integer)
        result = value.fetch(key)
      else
        return nil
      end

      if result.is_a?(Proc) && value.respond_to?(:[]=)
        result = result.arity == 0 ? result.call : result.call(self)
        value[key] = result
      end

      result = result.to_liquid if result.respond_to?(:to_liquid)
      result.context = self if result.respond_to?(:context=)
      result
    end

    def try_variable_find_in_environments(key, raise_on_not_found:)
      @environments.each do |environment|
        found_variable = lookup_and_evaluate(environment, key, raise_on_not_found: raise_on_not_found)
        return found_variable if !found_variable.nil? || @strict_variables && raise_on_not_found
      end

      @static_environments.each do |environment|
        found_variable = lookup_and_evaluate(environment, key, raise_on_not_found: raise_on_not_found)
        return found_variable if !found_variable.nil? || @strict_variables && raise_on_not_found
      end

      nil
    end

    def native_scope_chain
      @static_environments.reverse + @environments.reverse + @scopes.reverse
    end

    def sync_native_scopes!
      assign_context_to_roots!(@scopes)
      assign_context_to_roots!(@environments)
      assign_context_to_roots!(@static_environments)
      @native_handle["scopes"] = native_scope_chain
    end

    def sync_native_metadata!
      @native_handle["context"] = self
      @native_handle["local_scopes"] = @scopes
      @native_handle["environments"] = @environments
      @native_handle["static_environments"] = @static_environments
      @native_handle["counter_assigns"] ||= {}
      @native_handle["strict_variables"] = @strict_variables
      @native_handle["strict_filters"] = @strict_filters
      @native_handle["filters"] = @filters
    end

    def live_scope_depth
      Liquid::RustExtension.ext_context_live_depth(@native_handle)
    end

    def wrap_live_root(key, value)
      return Liquid::ForloopDrop.from_snapshot(value) if key == "forloop"

      value
    end

    def check_overflow
      raise StackLevelError, "Nesting too deep" if overflow?
    end

    def overflow?
      base_scope_depth + @scopes.length > Block::MAX_DEPTH
    end

    def internal_error
      raise Liquid::InternalError, "internal"
    rescue Liquid::InternalError => error
      error
    end

    def squash_instance_assigns_with_environments
      @scopes.last.each_key do |key|
        @environments.each do |environment|
          next unless environment.key?(key)

          @scopes.last[key] = lookup_and_evaluate(environment, key)
          break
        end
      end
    end

    def assign_context_to_roots!(collections)
      seen = {}
      collections.each do |scope|
        next unless scope.is_a?(Hash)

        scope.each_value do |value|
          assign_context_recursively(value, seen)
        end
      end
    end

    def assign_context_recursively(value, seen = {})
      if value.is_a?(Array) || value.is_a?(Hash)
        object_id = value.object_id
        return if seen[object_id]

        seen[object_id] = true
      end

      value.context = self if value.respond_to?(:context=)

      case value
      when Array
        value.each { |item| assign_context_recursively(item, seen) }
      when Hash
        value.each_value { |item| assign_context_recursively(item, seen) }
      end
    end
  end
end
