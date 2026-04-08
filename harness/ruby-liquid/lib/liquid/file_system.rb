# frozen_string_literal: true

module Liquid
  class LocalFileSystem
    attr_accessor :root

    def initialize(root, pattern = "_%s.liquid")
      @root = root
      @pattern = pattern
    end

    def read_template_file(template_path)
      full_path = full_path(template_path)
      raise FileSystemError, "No such template '#{template_path}'" unless File.exist?(full_path)

      File.read(full_path)
    end

    def full_path(template_path)
      unless %r{\A[^./][a-zA-Z0-9_/]*\z}.match?(template_path)
        raise FileSystemError, "Illegal template name '#{template_path}'"
      end

      full_path =
        if template_path.include?("/")
          File.join(root, File.dirname(template_path), @pattern % File.basename(template_path))
        else
          File.join(root, @pattern % template_path)
        end

      expanded_root = File.expand_path(root)
      expanded_path = File.expand_path(full_path)
      path_separators = [File::SEPARATOR, File::ALT_SEPARATOR].compact.uniq
      root_prefixes = path_separators.map { |separator| "#{expanded_root}#{separator}" }
      unless expanded_path == expanded_root || root_prefixes.any? { |prefix| expanded_path.start_with?(prefix) }
        raise FileSystemError, "Illegal template path '#{expanded_path}'"
      end

      full_path
    end
  end
end
