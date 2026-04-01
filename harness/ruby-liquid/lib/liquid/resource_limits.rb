# frozen_string_literal: true

module Liquid
  class ResourceLimits
    attr_accessor :render_length_limit, :render_score_limit, :assign_score_limit

    def initialize(render_length_limit: nil, render_score_limit: nil, assign_score_limit: nil)
      @render_length_limit = render_length_limit
      @render_score_limit = render_score_limit
      @assign_score_limit = assign_score_limit
    end

    def reset; end
  end
end
