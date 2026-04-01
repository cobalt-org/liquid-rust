# frozen_string_literal: true

module Liquid
  class BlankFileSystem
    def read_template_file(_template_path)
      raise Liquid::FileSystemError, "This liquid context does not allow includes."
    end
  end

  class Environment
    attr_accessor :exception_renderer, :default_resource_limits
    attr_reader :error_mode, :file_system, :tags, :filters

    class << self
      def default
        @default ||= new
      end

      def build(**options)
        env = default.dup
        options.each do |key, value|
          setter = :"#{key}="
          env.public_send(setter, value) if env.respond_to?(setter)
        end
        yield env if block_given?
        env
      end

      def dangerously_override(environment)
        previous = @default
        @default = environment
        yield
      ensure
        @default = previous
      end
    end

    def initialize(error_mode: :strict, file_system: BlankFileSystem.new, tags: {}, filters: [], exception_renderer: nil)
      @error_mode = error_mode
      @file_system = file_system
      @tags = tags.dup
      @filters = filters.dup
      @exception_renderer = exception_renderer
      @default_resource_limits = ResourceLimits.new
      @native_handle = Liquid::RustExtension.ext_env_build(
        "error_mode" => @error_mode.to_s,
        "file_system" => @file_system,
        "tags" => @tags,
        "filters" => @filters
      )
    end

    def dup
      copy = self.class.new(
        error_mode: @error_mode,
        file_system: @file_system,
        tags: @tags,
        filters: @filters,
        exception_renderer: @exception_renderer
      )
      copy.default_resource_limits = ResourceLimits.new(@default_resource_limits)
      copy
    end

    def error_mode=(error_mode)
      @error_mode = error_mode
      @native_handle&.[]=("error_mode", @error_mode.to_s)
    end

    def file_system=(file_system)
      @file_system = file_system
      @native_handle&.[]=("file_system", @file_system)
    end

    def register_tag(name, klass)
      @tags[name.to_s] = klass
      Liquid::RustExtension.ext_env_register_tag(@native_handle, name.to_s, klass)
      nil
    end

    def register_filter(mod)
      @filters << mod
      Liquid::RustExtension.ext_env_register_filter(@native_handle, mod)
      nil
    end

    def register_filters(modules)
      Array(modules).each { |mod| register_filter(mod) }
    end

    def tag_for_name(name)
      @tags[name.to_s]
    end

    def strainer_template
      nil
    end

    def native_handle
      @native_handle
    end
  end
end
