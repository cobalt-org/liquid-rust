# frozen_string_literal: true

module Liquid
  class BlankFileSystem
    def read_template_file(_template_path)
      raise Liquid::FileSystemError, "This liquid context does not allow includes."
    end
  end

  class Environment
    DEFAULT_MUTEX = Mutex.new

    class TagRegistry < Hash
      def initialize(environment, initial = {})
        @environment = environment
        super()
        merge!(initial.transform_keys(&:to_s))
      end

      def []=(key, value)
        super(key.to_s, value).tap do
          @environment.sync_tag_registration(key.to_s, value)
        end
      end

      def merge!(other)
        other.each { |key, value| self[key] = value }
        self
      end
    end

    attr_accessor :tags, :strainer_template, :exception_renderer, :default_resource_limits
    attr_reader :error_mode, :file_system, :filters

    class << self
      def default
        DEFAULT_MUTEX.synchronize do
          @default ||= new
        end
      end

      def build(**options)
        env = DEFAULT_MUTEX.synchronize do
          (@default ||= new).dup
        end
        options.each do |key, value|
          setter = :"#{key}="
          env.public_send(setter, value) if env.respond_to?(setter)
        end
        yield env if block_given?
        env
      end

      def dangerously_override(environment)
        previous = DEFAULT_MUTEX.synchronize do
          previous = @default
          @default = environment
          previous
        end
        yield
      ensure
        DEFAULT_MUTEX.synchronize do
          @default = previous
        end
      end
    end

    def initialize(error_mode: :lax, file_system: BlankFileSystem.new, tags: {}, filters: [], exception_renderer: nil)
      @tags = TagRegistry.new(self)
      @error_mode = error_mode
      @strainer_template = Class.new(StrainerTemplate).tap do |klass|
        klass.add_filter(StandardFilters)
      end
      @exception_renderer = exception_renderer || ->(exception) { exception }
      @file_system = file_system
      @default_resource_limits = ResourceLimits.new
      @strainer_template_class_cache = {}
      @filters = [StandardFilters]
      @tags.merge!(tags)
      register_filters(filters)
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
      nil
    end

    def register_filter(mod)
      @filters << mod
      @strainer_template_class_cache.clear
      @strainer_template.add_filter(mod)
      Liquid::RustExtension.ext_env_register_filter(@native_handle, mod) if @native_handle
      nil
    end

    def register_filters(modules)
      Array(modules).each { |mod| register_filter(mod) }
      self
    end

    def create_strainer(context, filters = [])
      return @strainer_template.new(context) if filters.empty?

      template = @strainer_template_class_cache[filters] ||= begin
        klass = Class.new(@strainer_template)
        filters.each { |filter| klass.add_filter(filter) }
        klass
      end

      template.new(context)
    end

    def filter_method_names
      @strainer_template.filter_method_names
    end

    def tag_for_name(name)
      @tags[name.to_s]
    end

    def native_handle
      @native_handle
    end

    def sync_tag_registration(name, klass)
      Liquid::RustExtension.ext_env_register_tag(@native_handle, name, klass) if @native_handle
    end
  end
end
