# frozen_string_literal: true

module LiquidHarnessBootstrap
  ROOT = File.expand_path(__dir__)
  GEM_ROOT = File.join(ROOT, "ruby-liquid")
  GEM_LIB = File.join(GEM_ROOT, "lib")
  LIQUID_ENTRYPOINT = File.join(GEM_LIB, "liquid.rb")
  PROFILER_ENTRYPOINT = File.join(GEM_LIB, "liquid", "profiler.rb")

  module_function

  def activate!
    return if $LOAD_PATH.first == GEM_LIB

    $LOAD_PATH.unshift(GEM_LIB)
  end

  def loaded?
    defined?(::Liquid::RUST_BACKED) && ::Liquid::RUST_BACKED
  end

  def verify!
    return true if loaded?

    raise LoadError, "expected the Rust-backed replacement gem to load, but upstream Shopify/liquid won the require race"
  end

  def mark_loaded!(*features)
    features.each do |feature|
      next if $LOADED_FEATURES.include?(feature)

      $LOADED_FEATURES << feature
    end
  end

  def preload!
    return if loaded?

    activate!
    require(LIQUID_ENTRYPOINT)
    mark_loaded!("liquid", "liquid.rb", LIQUID_ENTRYPOINT)
    require(PROFILER_ENTRYPOINT)
    mark_loaded!("liquid/profiler", "liquid/profiler.rb", PROFILER_ENTRYPOINT)
    verify!
  end

  def liquid_feature?(feature)
    return true if feature == "liquid" || feature == "liquid.rb"

    feature.end_with?("/liquid.rb", "\\liquid.rb")
  end

  def profiler_feature?(feature)
    return true if feature == "liquid/profiler"

    feature.end_with?("/liquid/profiler.rb", "\\liquid\\profiler.rb", "/liquid/profiler", "\\liquid\\profiler")
  end
end

module LiquidHarnessBootstrapRequirePatch
  def require(feature)
    LiquidHarnessBootstrap.activate!

    if LiquidHarnessBootstrap.liquid_feature?(feature)
      return false if LiquidHarnessBootstrap.loaded?

      result = super(LiquidHarnessBootstrap::LIQUID_ENTRYPOINT)
      LiquidHarnessBootstrap.verify!
      result
    elsif LiquidHarnessBootstrap.profiler_feature?(feature)
      require("liquid") unless LiquidHarnessBootstrap.loaded?
      super(LiquidHarnessBootstrap::PROFILER_ENTRYPOINT)
    else
      super(feature)
    end
  end
end

Kernel.prepend(LiquidHarnessBootstrapRequirePatch) unless Kernel.ancestors.include?(LiquidHarnessBootstrapRequirePatch)
LiquidHarnessBootstrap.activate!
LiquidHarnessBootstrap.preload!
