# frozen_string_literal: true

module Liquid
  class ResourceLimits
    attr_accessor :render_length_limit,
      :render_score_limit,
      :assign_score_limit,
      :cumulative_render_score_limit,
      :cumulative_assign_score_limit
    attr_reader :render_score,
      :assign_score,
      :cumulative_render_score,
      :cumulative_assign_score

    def initialize(limits = nil, **legacy_limits)
      limits =
        case limits
        when ResourceLimits
          {
            render_length_limit: limits.render_length_limit,
            render_score_limit: limits.render_score_limit,
            assign_score_limit: limits.assign_score_limit,
            cumulative_render_score_limit: limits.cumulative_render_score_limit,
            cumulative_assign_score_limit: limits.cumulative_assign_score_limit
          }
        when Hash
          limits.transform_keys(&:to_sym)
        else
          {}
        end

      limits = limits.merge(legacy_limits.transform_keys(&:to_sym))

      @render_length_limit = limits[:render_length_limit]
      @render_score_limit = limits[:render_score_limit]
      @assign_score_limit = limits[:assign_score_limit]
      @cumulative_render_score_limit = limits[:cumulative_render_score_limit]
      @cumulative_assign_score_limit = limits[:cumulative_assign_score_limit]
      @cumulative_render_score = 0
      @cumulative_assign_score = 0
      reset
    end

    def increment_render_score(amount)
      @render_score += amount
      @cumulative_render_score += amount
      raise_limits_reached if @render_score_limit && @render_score > @render_score_limit
      raise_limits_reached if @cumulative_render_score_limit && @cumulative_render_score > @cumulative_render_score_limit
    end

    def increment_assign_score(amount)
      @assign_score += amount
      @cumulative_assign_score += amount
      raise_limits_reached if @assign_score_limit && @assign_score > @assign_score_limit
      raise_limits_reached if @cumulative_assign_score_limit && @cumulative_assign_score > @cumulative_assign_score_limit
    end

    def increment_write_score(output_or_bytes)
      bytes =
        case output_or_bytes
        when Integer
          output_or_bytes
        else
          output_or_bytes.to_s.bytesize
        end

      if (last_captured = @last_capture_length)
        increment = bytes - last_captured
        @last_capture_length = bytes
        increment_assign_score(increment)
      elsif @render_length_limit && bytes > @render_length_limit
        raise_limits_reached
      end
    end

    def raise_limits_reached
      @reached_limit = true
      raise MemoryError, "Memory limits exceeded"
    end

    def reached?
      @reached_limit
    end

    def reset
      @reached_limit = false
      @last_capture_length = nil
      @render_score = 0
      @assign_score = 0
      raise_limits_reached if @cumulative_render_score_limit && @cumulative_render_score > @cumulative_render_score_limit
      raise_limits_reached if @cumulative_assign_score_limit && @cumulative_assign_score > @cumulative_assign_score_limit
    end

    def with_capture
      old_capture_length = @last_capture_length
      begin
        @last_capture_length = 0
        yield
      ensure
        @last_capture_length = old_capture_length
      end
    end
  end
end
