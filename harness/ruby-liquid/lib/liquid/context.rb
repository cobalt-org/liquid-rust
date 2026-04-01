# frozen_string_literal: true

module Liquid
  class Context
    attr_accessor :environment, :exception_renderer, :template_name, :global_filter
    attr_reader :registers, :environments, :errors, :warnings, :resource_limits

    def self.build(environments: [], static_environments: {}, rethrow_errors: false, registers: nil, environment: nil)
      scopes = Array(environments)
      scopes << static_environments unless static_environments.nil? || static_environments.empty?
      new(scopes, rethrow_errors: rethrow_errors, registers: registers, environment: environment)
    end

    def initialize(
      environments = {},
      rethrow_errors: false,
      registers: nil,
      environment: nil,
      resource_limits: nil,
      **legacy_environments
    )
      if legacy_environments.any?
        keyword_environment = legacy_environments.transform_keys(&:to_s)
        environments =
          if environments.is_a?(Hash)
            keyword_environment.merge(environments)
          else
            [keyword_environment, environments].flatten(1)
          end
      end

      @environment = environment || Liquid::Environment.default
      @rethrow_errors = rethrow_errors
      @registers = registers.is_a?(Liquid::Registers) ? registers : Liquid::Registers.new(registers || {})
      @resource_limits = resource_limits || ResourceLimits.new(@environment.default_resource_limits)
      @errors = []
      @warnings = []
      @filters = []
      @global_filter = nil
      @strict_variables = false
      @strict_filters = false
      @environments =
        case environments
        when Array
          environments
        when Hash
          [environments]
        else
          [environments || {}]
        end
      @native_handle = Liquid::RustExtension.ext_context_new(@environments, @registers.to_h, @environment.error_mode.to_s)
      @native_handle["context"] = self
      @native_handle["strict_variables"] = @strict_variables
      @native_handle["strict_filters"] = @strict_filters
      @native_handle["filters"] = @filters
    end

    def strict_variables
      @strict_variables
    end

    def strict_variables=(value)
      @strict_variables = value
      @native_handle["strict_variables"] = value if defined?(@native_handle) && @native_handle
    end

    def strict_filters
      @strict_filters
    end

    def strict_filters=(value)
      @strict_filters = value
      @native_handle["strict_filters"] = value if defined?(@native_handle) && @native_handle
    end

    def [](key)
      find_variable(key.to_s)
    end

    def []=(key, value)
      Liquid::RustExtension.ext_context_set(@native_handle, key.to_s, value)
    end

    def key?(key)
      Liquid::RustExtension.ext_context_has_key(@native_handle, key.to_s)
    end

    def add_filters(filter_module)
      @filters.concat(Array(filter_module))
      @native_handle["filters"] = @filters if defined?(@native_handle) && @native_handle
      nil
    end

    def push(new_scope = {})
      @environments << new_scope.dup
      Liquid::RustExtension.ext_context_push(@native_handle, new_scope)
      nil
    end

    def pop
      @environments.pop
      Liquid::RustExtension.ext_context_pop(@native_handle)
      nil
    end

    def stack
      push({})
      yield
    ensure
      pop
    end

    def find_variable(key)
      key = key.to_s
      if @strict_variables && !Liquid::RustExtension.ext_context_has_key(@native_handle, key)
        raise Liquid::UndefinedVariable, "undefined variable #{key}"
      end

      Liquid::RustExtension.ext_context_find_variable(@native_handle, key)
    end

    def apply_global_filter(output)
      return output unless @global_filter

      @global_filter.call(output)
    end

    def to_liquid_payload
      @environments.each_with_object({}) do |scope, merged|
        next merged unless scope.is_a?(Hash)

        merged.merge!(scope)
      end
    end

    def native_handle
      @native_handle
    end
  end
end
